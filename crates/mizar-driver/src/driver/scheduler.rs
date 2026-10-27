use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use mizar_build::{
    scheduler::{
        SchedulerDiagnosticRef, SchedulerDispatchOutcome, SchedulerDispatchTask, SchedulerRun,
        SchedulerTaskDispatcher, TaskState,
    },
    task_graph::{BuildTask, PipelinePhase, TaskGraph, TaskId},
};
use mizar_ir::{
    dispatch_input::{PhaseDispatchInputProvider, PhaseDispatchInputRequest},
    publisher::PhaseOutputPublisher,
};

use crate::{
    driver::DriverSchedulerRun,
    registry::{
        PhaseExecutionResources, PhaseInput, PhaseRegistry, PhaseRegistryError, PhaseResult,
        PhaseStatus, SourceLoadInputs,
    },
    request::BuildSessionOutcome,
};

impl DriverSchedulerRun {
    pub(super) fn from_scheduler_run(run: SchedulerRun) -> Self {
        Self {
            phase_results: BTreeMap::new(),
            task_states: run.task_states,
            events: run.events,
            diagnostics: run.diagnostics,
        }
    }
}

pub(super) fn scheduler_outcome(run: &SchedulerRun) -> BuildSessionOutcome {
    if run
        .task_states
        .iter()
        .any(|record| record.state == TaskState::Failed)
    {
        BuildSessionOutcome::Failed
    } else if run
        .task_states
        .iter()
        .any(|record| record.state == TaskState::Blocked)
    {
        BuildSessionOutcome::Blocked
    } else if run
        .task_states
        .iter()
        .any(|record| record.state == TaskState::Cancelled)
    {
        BuildSessionOutcome::Cancelled
    } else {
        BuildSessionOutcome::Succeeded
    }
}

pub(super) struct RegistrySchedulerDispatcher<'a> {
    registry: &'a PhaseRegistry,
    phase_inputs: Option<&'a dyn PhaseDispatchInputProvider<BuildTask>>,
    publisher: Option<&'a Arc<PhaseOutputPublisher>>,
    source_load: Option<SourceLoadInputs<'a>>,
    pub(super) phase_results: BTreeMap<TaskId, Vec<PhaseResult>>,
}

impl<'a> RegistrySchedulerDispatcher<'a> {
    pub(super) const fn new(
        registry: &'a PhaseRegistry,
        phase_inputs: Option<&'a dyn PhaseDispatchInputProvider<BuildTask>>,
        publisher: Option<&'a Arc<PhaseOutputPublisher>>,
        source_load: Option<SourceLoadInputs<'a>>,
    ) -> Self {
        Self {
            registry,
            phase_inputs,
            publisher,
            source_load,
            phase_results: BTreeMap::new(),
        }
    }
}

impl SchedulerTaskDispatcher for RegistrySchedulerDispatcher<'_> {
    fn dispatch(&mut self, task: SchedulerDispatchTask<'_>) -> SchedulerDispatchOutcome {
        dispatch_registry_phase(
            self.registry,
            self.phase_inputs,
            self.publisher,
            self.source_load,
            &mut self.phase_results,
            task,
        )
    }
}

pub(super) fn dispatch_gap_phases(
    task_graph: &TaskGraph,
    run: &SchedulerRun,
) -> Vec<PipelinePhase> {
    let dispatch_blocked_tasks = run
        .results
        .iter()
        .filter(|result| {
            result.state == TaskState::Blocked
                && result
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == "missing_phase_input_identities")
        })
        .map(|record| record.task_id.clone())
        .collect::<BTreeSet<_>>();
    let mut phases = BTreeMap::new();
    for task in task_graph.tasks() {
        if dispatch_blocked_tasks.contains(&task.id) {
            for phase in &task.phases {
                if *phase != PipelinePhase::PackageResolve {
                    phases.entry(*phase).or_insert(());
                }
            }
        }
    }
    phases.into_keys().collect()
}

fn dispatch_registry_phase(
    registry: &PhaseRegistry,
    phase_inputs: Option<&dyn PhaseDispatchInputProvider<BuildTask>>,
    publisher: Option<&Arc<PhaseOutputPublisher>>,
    source_load: Option<SourceLoadInputs<'_>>,
    phase_results: &mut BTreeMap<TaskId, Vec<PhaseResult>>,
    task: SchedulerDispatchTask<'_>,
) -> SchedulerDispatchOutcome {
    if task.task.phases.is_empty() {
        return SchedulerDispatchOutcome::blocked(vec![dispatch_diagnostic(
            task.task,
            "missing_phase",
            "scheduler-selected task has no registry phase",
        )]);
    }
    if task
        .task
        .phases
        .iter()
        .all(|phase| *phase == PipelinePhase::PackageResolve)
    {
        return SchedulerDispatchOutcome::complete();
    }

    let mut type_prefix = false;
    let mut registration_prefix = false;
    let mut overload_prefix = false;
    let mut elaboration_prefix = false;
    let mut preparation_prefix = false;
    let mut vc_group = false;
    let dispatch_input = match phase_inputs {
        Some(provider) => {
            match provider
                .dispatch_input_for_task(PhaseDispatchInputRequest::new(task.task, task.snapshot))
            {
                Ok(Some(bundle)) => {
                    if let Err(error) = bundle.validate_snapshot(task.snapshot) {
                        return SchedulerDispatchOutcome::failed(vec![dispatch_diagnostic(
                            task.task,
                            "invalid_phase_dispatch_input",
                            error.to_string(),
                        )]);
                    }
                    bundle
                }
                Ok(None) => {
                    return SchedulerDispatchOutcome::blocked(vec![dispatch_diagnostic(
                        task.task,
                        "missing_phase_input_identities",
                        "owner-provided phase input bundle is unavailable",
                    )]);
                }
                Err(error) => {
                    return SchedulerDispatchOutcome::failed(vec![dispatch_diagnostic(
                        task.task,
                        "invalid_phase_dispatch_input",
                        error.to_string(),
                    )]);
                }
            }
        }
        None => {
            use crate::{
                frontend_adapter::{
                    FrontendService, SourceLoadService, SourceTheoremAlgorithmPreparationService,
                    SourceTheoremArtifactCommitService, SourceTheoremAtpSolveService,
                    SourceTheoremCandidateCheckService, SourceTheoremElaborationService,
                    SourceTheoremModuleResolverService, SourceTheoremOverloadResolutionService,
                    SourceTheoremRegistrationResolutionService,
                    SourceTheoremSignatureCollectionService, SourceTheoremTypeCheckingService,
                    SourceTheoremVcDischargeService, SourceTheoremVcGenerationService,
                },
                registry::PhaseService,
            };
            let check_and_elaborate = task.task.kind
                == mizar_build::task_graph::TaskKind::CheckAndElaborate
                && task.task.phases
                    == [
                        PipelinePhase::SignatureCollection,
                        PipelinePhase::TypeChecking,
                        PipelinePhase::RegistrationResolution,
                        PipelinePhase::OverloadResolution,
                        PipelinePhase::Elaboration,
                        PipelinePhase::AlgorithmPreparation,
                    ];
            let vc_task = task.task.kind == mizar_build::task_graph::TaskKind::VcGenerate
                && task.task.phases == [PipelinePhase::VcGenerate];
            let discharge_task = task.task.kind == mizar_build::task_graph::TaskKind::VcDischarge
                && task.task.phases == [PipelinePhase::VcDischarge]
                && matches!(task.task.unit, mizar_build::task_graph::WorkUnit::Vc { .. });
            let atp_task = task.task.kind == mizar_build::task_graph::TaskKind::AtpSolve
                && task.task.phases == [PipelinePhase::AtpSolve]
                && matches!(task.task.unit, mizar_build::task_graph::WorkUnit::Vc { .. });
            let kernel_task = task.task.kind == mizar_build::task_graph::TaskKind::KernelCheck
                && task.task.phases == [PipelinePhase::KernelCheck]
                && matches!(
                    task.task.unit,
                    mizar_build::task_graph::WorkUnit::EvidenceCandidate { .. }
                );
            let artifact_task = task.task.kind == mizar_build::task_graph::TaskKind::ArtifactCommit
                && task.task.phases == [PipelinePhase::ArtifactCommit]
                && matches!(
                    task.task.unit,
                    mizar_build::task_graph::WorkUnit::Module { .. }
                );
            let artifact_service = registry.descriptors().len() == 14
                && registry
                    .service_for_phase(PipelinePhase::ArtifactCommit)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremArtifactCommitService>()
                            && registry
                                .descriptor_for_phase(PipelinePhase::ArtifactCommit)
                                .ok()
                                == Some(&service.phase())
                    })
                && [
                    (
                        PipelinePhase::SourceLoad,
                        std::any::TypeId::of::<SourceLoadService>(),
                    ),
                    (
                        PipelinePhase::Frontend,
                        std::any::TypeId::of::<FrontendService>(),
                    ),
                ]
                .into_iter()
                .all(|(phase, concrete)| {
                    registry
                        .service_for_phase(phase)
                        .is_ok_and(|service| std::any::Any::type_id(service) == concrete)
                });
            let kernel_service = (registry.descriptors().len() == 13 || artifact_service)
                && registry
                    .service_for_phase(PipelinePhase::KernelCheck)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremCandidateCheckService>()
                            && registry
                                .descriptor_for_phase(PipelinePhase::KernelCheck)
                                .ok()
                                == Some(&service.phase())
                    });
            let atp_service = (registry.descriptors().len() == 12 || kernel_service)
                && registry
                    .service_for_phase(PipelinePhase::AtpSolve)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremAtpSolveService>()
                            && registry.descriptor_for_phase(PipelinePhase::AtpSolve).ok()
                                == Some(&service.phase())
                    })
                && matches!(
                    registry.descriptor_for_phase(PipelinePhase::BackendRun),
                    Err(PhaseRegistryError::MissingPhaseService { .. })
                );
            let signature_prefix = check_and_elaborate
                && registry
                    .descriptor_for_phase(PipelinePhase::SignatureCollection)
                    .ok()
                    == Some(&SourceTheoremSignatureCollectionService.phase())
                && registry
                    .service_for_phase(PipelinePhase::SignatureCollection)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremSignatureCollectionService>()
                    })
                && matches!(
                    registry.descriptor_for_phase(PipelinePhase::TypeChecking),
                    Err(PhaseRegistryError::MissingPhaseService { .. })
                );
            let typed_profile = (check_and_elaborate
                || vc_task
                || discharge_task
                || atp_task
                || kernel_task
                || artifact_task)
                && registry
                    .descriptor_for_phase(PipelinePhase::SourceLoad)
                    .ok()
                    == Some(&SourceLoadService.phase())
                && registry.descriptor_for_phase(PipelinePhase::Frontend).ok()
                    == Some(
                        &FrontendService {
                            artifact_roots: Vec::new(),
                        }
                        .phase(),
                    )
                && [
                    (
                        PipelinePhase::ModuleResolve,
                        SourceTheoremModuleResolverService.phase(),
                        std::any::TypeId::of::<SourceTheoremModuleResolverService>(),
                    ),
                    (
                        PipelinePhase::SignatureCollection,
                        SourceTheoremSignatureCollectionService.phase(),
                        std::any::TypeId::of::<SourceTheoremSignatureCollectionService>(),
                    ),
                    (
                        PipelinePhase::TypeChecking,
                        SourceTheoremTypeCheckingService.phase(),
                        std::any::TypeId::of::<SourceTheoremTypeCheckingService>(),
                    ),
                ]
                .into_iter()
                .all(|(phase, descriptor, concrete)| {
                    registry.descriptor_for_phase(phase).ok() == Some(&descriptor)
                        && registry
                            .service_for_phase(phase)
                            .is_ok_and(|service| std::any::Any::type_id(service) == concrete)
                });
            type_prefix = check_and_elaborate
                && typed_profile
                && registry.descriptors().len() == 5
                && matches!(
                    registry.descriptor_for_phase(PipelinePhase::RegistrationResolution),
                    Err(PhaseRegistryError::MissingPhaseService { .. })
                );
            let registration_profile = typed_profile
                && registry
                    .descriptor_for_phase(PipelinePhase::RegistrationResolution)
                    .ok()
                    == Some(&SourceTheoremRegistrationResolutionService.phase())
                && registry
                    .service_for_phase(PipelinePhase::RegistrationResolution)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremRegistrationResolutionService>()
                    });
            registration_prefix = check_and_elaborate
                && registration_profile
                && registry.descriptors().len() == 6
                && matches!(
                    registry.descriptor_for_phase(PipelinePhase::OverloadResolution),
                    Err(PhaseRegistryError::MissingPhaseService { .. })
                );
            let overload_profile = registration_profile
                && registry
                    .descriptor_for_phase(PipelinePhase::OverloadResolution)
                    .ok()
                    == Some(&SourceTheoremOverloadResolutionService.phase())
                && registry
                    .service_for_phase(PipelinePhase::OverloadResolution)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremOverloadResolutionService>()
                    });
            overload_prefix = check_and_elaborate
                && overload_profile
                && registry.descriptors().len() == 7
                && matches!(
                    registry.descriptor_for_phase(PipelinePhase::Elaboration),
                    Err(PhaseRegistryError::MissingPhaseService { .. })
                );
            let elaboration_profile = overload_profile
                && registry
                    .descriptor_for_phase(PipelinePhase::Elaboration)
                    .ok()
                    == Some(&SourceTheoremElaborationService.phase())
                && registry
                    .service_for_phase(PipelinePhase::Elaboration)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremElaborationService>()
                    });
            elaboration_prefix = check_and_elaborate
                && elaboration_profile
                && registry.descriptors().len() == 8
                && matches!(
                    registry.descriptor_for_phase(PipelinePhase::AlgorithmPreparation),
                    Err(PhaseRegistryError::MissingPhaseService { .. })
                );
            let preparation_profile = elaboration_profile
                && registry
                    .descriptor_for_phase(PipelinePhase::AlgorithmPreparation)
                    .ok()
                    == Some(&SourceTheoremAlgorithmPreparationService.phase())
                && registry
                    .service_for_phase(PipelinePhase::AlgorithmPreparation)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremAlgorithmPreparationService>()
                    });
            preparation_prefix = check_and_elaborate
                && preparation_profile
                && registry.descriptors().len() == 9
                && matches!(
                    registry.descriptor_for_phase(PipelinePhase::VcGenerate),
                    Err(PhaseRegistryError::MissingPhaseService { .. })
                );
            let vc_profile = preparation_profile
                && registry
                    .service_for_phase(PipelinePhase::VcGenerate)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremVcGenerationService>()
                            && registry
                                .descriptor_for_phase(PipelinePhase::VcGenerate)
                                .ok()
                                == Some(&service.phase())
                    })
                && (artifact_service
                    || matches!(
                        registry.descriptor_for_phase(PipelinePhase::ArtifactCommit),
                        Err(PhaseRegistryError::MissingPhaseService { .. })
                    ));
            let discharge_profile = vc_profile
                && (registry.descriptors().len() == 11 || atp_service)
                && registry
                    .service_for_phase(PipelinePhase::VcDischarge)
                    .is_ok_and(|service| {
                        std::any::Any::type_id(service)
                            == std::any::TypeId::of::<SourceTheoremVcDischargeService>()
                            && registry
                                .descriptor_for_phase(PipelinePhase::VcDischarge)
                                .ok()
                                == Some(&service.phase())
                    });
            let vc_prefix = (check_and_elaborate || vc_task)
                && vc_profile
                && (registry.descriptors().len() == 10 || discharge_profile);
            let discharge_prefix = discharge_task && discharge_profile;
            let atp_prefix = atp_task && discharge_profile && atp_service;
            let kernel_prefix = kernel_task && discharge_profile && atp_service && kernel_service;
            let artifact_prefix = artifact_task
                && discharge_profile
                && atp_service
                && kernel_service
                && artifact_service;
            let candidate_prefix = atp_prefix || kernel_prefix || artifact_prefix;
            vc_group = vc_prefix && check_and_elaborate;
            if !signature_prefix
                && !type_prefix
                && !registration_prefix
                && !overload_prefix
                && !elaboration_prefix
                && !preparation_prefix
                && !vc_prefix
                && !discharge_prefix
                && !atp_prefix
                && !kernel_prefix
                && task.task.phases.iter().any(|phase| {
                    matches!(
                        registry.descriptor_for_phase(*phase),
                        Err(PhaseRegistryError::MissingPhaseService { .. })
                    )
                })
            {
                return SchedulerDispatchOutcome::blocked(vec![dispatch_diagnostic(
                    task.task,
                    "missing_phase_service",
                    "phase service is unavailable at dispatch time",
                )]);
            }
            let bundle = (|| {
                use crate::{
                    frontend_adapter::{
                        FrontendService, SourceLoadService, SourceTheoremModuleResolverService,
                    },
                    registry::PhaseService,
                };
                use mizar_ir::{
                    dispatch_input::{PhaseDispatchInputBundle, SealedParentOutputHandle},
                    identity::{OutputKind, PipelinePhase as IrPhase, WorkUnit as IrWorkUnit},
                };
                let expected = match task.task.phases.as_slice() {
                    [PipelinePhase::SourceLoad] => SourceLoadService.phase(),
                    [PipelinePhase::Frontend] => FrontendService {
                        artifact_roots: Vec::new(),
                    }
                    .phase(),
                    [PipelinePhase::VcGenerate] if vc_prefix => registry
                        .descriptor_for_phase(PipelinePhase::VcGenerate)
                        .ok()?
                        .clone(),
                    [PipelinePhase::VcDischarge] if discharge_prefix => registry
                        .descriptor_for_phase(PipelinePhase::VcDischarge)
                        .ok()?
                        .clone(),
                    [PipelinePhase::AtpSolve] if atp_prefix => registry
                        .descriptor_for_phase(PipelinePhase::AtpSolve)
                        .ok()?
                        .clone(),
                    [PipelinePhase::KernelCheck] if kernel_prefix => registry
                        .descriptor_for_phase(PipelinePhase::KernelCheck)
                        .ok()?
                        .clone(),
                    [PipelinePhase::ArtifactCommit] if artifact_prefix => registry
                        .descriptor_for_phase(PipelinePhase::ArtifactCommit)
                        .ok()?
                        .clone(),
                    [PipelinePhase::ModuleResolve] => {
                        if std::any::Any::type_id(
                            registry
                                .service_for_phase(PipelinePhase::ModuleResolve)
                                .ok()?,
                        ) != std::any::TypeId::of::<SourceTheoremModuleResolverService>()
                        {
                            return None;
                        }
                        SourceTheoremModuleResolverService.phase()
                    }
                    _ if signature_prefix
                        || type_prefix
                        || registration_prefix
                        || overload_prefix
                        || elaboration_prefix
                        || preparation_prefix
                        || vc_group =>
                    {
                        SourceTheoremSignatureCollectionService.phase()
                    }
                    _ => return None,
                };
                if registry.descriptor_for_phase(task.task.phases[0]).ok()? != &expected {
                    return None;
                }
                let publisher = publisher?;
                publisher.validate_current_snapshot(task.snapshot).ok()?;
                let inputs = source_load?;
                if inputs.snapshot.id != task.snapshot {
                    return None;
                }
                let module = match &task.task.unit {
                    mizar_build::task_graph::WorkUnit::Module { module } => module,
                    mizar_build::task_graph::WorkUnit::Vc { module, .. }
                        if discharge_prefix || atp_prefix =>
                    {
                        module
                    }
                    mizar_build::task_graph::WorkUnit::EvidenceCandidate { module, .. }
                        if kernel_prefix =>
                    {
                        module
                    }
                    _ => return None,
                };
                let mut versions = inputs.snapshot.source_versions.iter().filter(|version| {
                    version.package_id == module.package && version.module_path == module.path
                });
                let version = versions.next()?;
                if versions.next().is_some() {
                    return None;
                }
                let key = crate::frontend_adapter::source_input_hash(version);
                if task.task.phases == [PipelinePhase::SourceLoad] {
                    return Some(PhaseDispatchInputBundle::without_parent_outputs(
                        task.snapshot,
                        key,
                        Vec::new(),
                    ));
                }
                let unit = IrWorkUnit::new(format!(
                    "{:?}:{:?}",
                    module.package.as_str(),
                    module.path.as_str()
                ));
                let named = if discharge_prefix || candidate_prefix {
                    vec![mizar_ir::identity::NamedInputHash {
                        name: "source".to_owned(),
                        domain: crate::frontend_adapter::SOURCE_UNIT_CACHE_KEY_VERSION.to_owned(),
                        digest: key,
                    }]
                } else {
                    Vec::new()
                };
                let direct = if discharge_prefix || candidate_prefix {
                    let (phase, kind) = if candidate_prefix {
                        ("VcDischarge", "SourceTheoremDischarge")
                    } else {
                        ("VcGenerate", "SourceTheoremVC")
                    };
                    let mut candidates = task.task.dependencies.iter().filter_map(|id| {
                        let [result] = phase_results.get(id)?.as_slice() else {
                            return None;
                        };
                        let [output] = result.output_refs.as_slice() else {
                            return None;
                        };
                        (result.status == PhaseStatus::Complete
                            && result.diagnostics.is_empty()
                            && output.phase() == &IrPhase::new(phase)
                            && output.output_kind() == &OutputKind::new(kind)
                            && output.snapshot() == task.snapshot
                            && output.work_unit() == &unit
                            && output.schema_version() == mizar_ir::storage::SchemaVersion::new(1)
                            && output.lineage().named_input_hashes == named)
                            .then_some(output)
                    });
                    let output = candidates.next()?;
                    if candidates.next().is_some() {
                        return None;
                    }
                    Some(output)
                } else {
                    None
                };
                let vc = if candidate_prefix {
                    let discharge = direct?;
                    if discharge.lineage().parents.len() != 2 {
                        return None;
                    }
                    let mut candidates = phase_results.values().filter_map(|results| {
                        let [result] = results.as_slice() else {
                            return None;
                        };
                        let [output] = result.output_refs.as_slice() else {
                            return None;
                        };
                        (result.status == PhaseStatus::Complete
                            && result.diagnostics.is_empty()
                            && output.phase() == &IrPhase::new("VcGenerate")
                            && output.output_kind() == &OutputKind::new("SourceTheoremVC")
                            && output.snapshot() == task.snapshot
                            && output.work_unit() == &unit
                            && output.schema_version() == mizar_ir::storage::SchemaVersion::new(1)
                            && output.lineage().named_input_hashes == named
                            && discharge.lineage().parents.contains(&output.output()))
                        .then_some(output)
                    });
                    let output = candidates.next()?;
                    if candidates.next().is_some() {
                        return None;
                    }
                    Some(output)
                } else {
                    direct
                };
                let parents = if (vc_task && vc_prefix) || discharge_prefix || candidate_prefix {
                    let mut dependencies = phase_results.iter().filter_map(|(id, results)| {
                        if !discharge_prefix
                            && !candidate_prefix
                            && !task.task.dependencies.contains(id)
                        {
                            return None;
                        }
                        let expected = [
                            ("SignatureCollection", "SourceTheoremModuleSummary"),
                            ("TypeChecking", "SourceTheoremTypedAst"),
                            ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                            ("OverloadResolution", "SourceTheoremMetadata"),
                            ("Elaboration", "SourceTheoremCore"),
                            ("AlgorithmPreparation", "SourceTheoremPreparation"),
                        ];
                        if results.len() != expected.len()
                            || !results.iter().zip(expected).all(|(result, (phase, kind))| {
                                let [output] = result.output_refs.as_slice() else {
                                    return false;
                                };
                                result.status == PhaseStatus::Complete
                                    && result.diagnostics.is_empty()
                                    && output.phase() == &IrPhase::new(phase)
                                    && output.output_kind() == &OutputKind::new(kind)
                                    && output.snapshot() == task.snapshot
                                    && output.work_unit() == &unit
                                    && output.schema_version()
                                        == mizar_ir::storage::SchemaVersion::new(1)
                                    && (!(discharge_prefix || candidate_prefix)
                                        || output.lineage().named_input_hashes == named)
                            })
                        {
                            return None;
                        }
                        let metadata = &results[3].output_refs[0];
                        let core = &results[4].output_refs[0];
                        let preparation = &results[5].output_refs[0];
                        (core.lineage().parents == vec![metadata.output()]
                            && preparation.lineage().parents == vec![core.output()]
                            && vc.is_none_or(|vc| {
                                vc.lineage().parents.len() == 2
                                    && vc.lineage().parents.contains(&core.output())
                                    && vc.lineage().parents.contains(&preparation.output())
                            })
                            && (!candidate_prefix
                                || direct?.lineage().parents.contains(&core.output())))
                        .then_some((&results[0].output_refs[0], core, preparation))
                    });
                    let (summary, core, preparation) = dependencies.next()?;
                    if dependencies.next().is_some() {
                        return None;
                    }
                    let mut parents = vec![core, vc.unwrap_or(preparation)];
                    if candidate_prefix {
                        parents.push(direct?);
                    }
                    if artifact_prefix {
                        let mut kernels = task.task.dependencies.iter().filter_map(|id| {
                            let [result] = phase_results.get(id)?.as_slice() else {
                                return None;
                            };
                            let [kernel, file] = result.output_refs.as_slice() else {
                                return None;
                            };
                            (result.status == PhaseStatus::Complete
                                && result.diagnostics.is_empty()
                                && [
                                    (kernel, "SourceTheoremCandidateCheck"),
                                    (file, "SourceTheoremCandidateWitnessFile"),
                                ]
                                .into_iter()
                                .all(|(output, kind)| {
                                    output.phase() == &IrPhase::new("KernelCheck")
                                        && output.output_kind() == &OutputKind::new(kind)
                                        && output.snapshot() == task.snapshot
                                        && output.work_unit() == &unit
                                        && output.schema_version()
                                            == mizar_ir::storage::SchemaVersion::new(1)
                                        && output.lineage().named_input_hashes == named
                                })
                                && kernel.lineage().parents.len() == 4
                                && [core.output(), vc?.output(), direct?.output()]
                                    .iter()
                                    .all(|id| kernel.lineage().parents.contains(id))
                                && file.lineage().parents == vec![kernel.output()])
                            .then_some((kernel, file))
                        });
                        let (kernel, file) = kernels.next()?;
                        if kernels.next().is_some() {
                            return None;
                        }
                        parents.insert(0, summary);
                        parents.extend([kernel, file]);
                    }
                    parents
                } else if task.task.phases == [PipelinePhase::ModuleResolve]
                    || signature_prefix
                    || type_prefix
                    || registration_prefix
                    || overload_prefix
                    || elaboration_prefix
                    || preparation_prefix
                    || vc_group
                {
                    let (parent_phase, parent_kind) = if signature_prefix
                        || type_prefix
                        || registration_prefix
                        || overload_prefix
                        || elaboration_prefix
                        || preparation_prefix
                        || vc_group
                    {
                        (
                            IrPhase::new("ModuleResolve"),
                            OutputKind::new("SourceTheoremModuleCandidates"),
                        )
                    } else {
                        (IrPhase::new("Frontend"), OutputKind::new("FrontendOutput"))
                    };
                    let mut parents = task.task.dependencies.iter().filter_map(|id| {
                        let [result] = phase_results.get(id)?.as_slice() else {
                            return None;
                        };
                        let [parent] = result.output_refs.as_slice() else {
                            return None;
                        };
                        (result.status == PhaseStatus::Complete
                            && result.diagnostics.is_empty()
                            && parent.phase() == &parent_phase
                            && parent.work_unit() == &unit
                            && parent.output_kind() == &parent_kind
                            && parent.schema_version() == mizar_ir::storage::SchemaVersion::new(1))
                        .then_some(parent)
                    });
                    let parent = parents.next()?;
                    if parents.next().is_some() {
                        return None;
                    }
                    vec![parent]
                } else {
                    let parents = task
                        .task
                        .dependencies
                        .iter()
                        .flat_map(|id| phase_results.get(id).into_iter().flatten())
                        .filter(|result| result.status == PhaseStatus::Complete)
                        .flat_map(|result| result.output_refs.iter())
                        .filter(|parent| {
                            (parent.phase() == &IrPhase::new("SourceLoad")
                                && parent.work_unit() == &unit
                                && parent.output_kind() == &OutputKind::new("SourceUnit"))
                                || (parent.phase() == &IrPhase::new("Frontend")
                                    && parent.output_kind() == &OutputKind::new("FrontendOutput"))
                        })
                        .collect::<Vec<_>>();
                    if parents
                        .iter()
                        .filter(|parent| parent.phase() == &IrPhase::new("SourceLoad"))
                        .count()
                        != 1
                    {
                        return None;
                    }
                    parents
                };
                let parents = parents
                    .into_iter()
                    .map(|parent| {
                        if inputs
                            .captured
                            .is_some_and(|capture| capture.native_sources().is_some())
                        {
                            SealedParentOutputHandle::from_published_output(
                                publisher,
                                task.snapshot,
                                parent.clone(),
                            )
                        } else {
                            SealedParentOutputHandle::from_current_output(
                                publisher,
                                task.snapshot,
                                parent.clone(),
                            )
                        }
                        .ok()
                    })
                    .collect::<Option<Vec<_>>>()?;
                let mut dependencies = inputs
                    .module_index
                    .dependency_summaries
                    .iter()
                    .map(|summary| summary.content_hash)
                    .collect::<Vec<_>>();
                dependencies.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
                PhaseDispatchInputBundle::new(task.snapshot, key, dependencies, parents).ok()
            })();
            let Some(bundle) = bundle else {
                return SchedulerDispatchOutcome::blocked(vec![dispatch_diagnostic(
                    task.task,
                    "missing_phase_input_identities",
                    "owner-provided phase input bundle is unavailable",
                )]);
            };
            bundle
        }
    };

    let mut dispatched_descriptors = BTreeSet::new();
    for phase in &task.task.phases {
        if *phase == PipelinePhase::PackageResolve {
            continue;
        }
        let descriptor = match registry.descriptor_for_phase(*phase) {
            Ok(descriptor) => descriptor,
            Err(PhaseRegistryError::MissingPhaseService { .. }) => {
                return SchedulerDispatchOutcome::blocked(vec![dispatch_diagnostic(
                    task.task,
                    "missing_phase_service",
                    "phase service is unavailable at dispatch time",
                )]);
            }
            Err(error) => {
                return SchedulerDispatchOutcome::failed(vec![dispatch_diagnostic(
                    task.task,
                    "phase_registry_error",
                    error.to_string(),
                )]);
            }
        };
        let descriptor_key = (
            descriptor.service_name.clone(),
            descriptor.schema_version.clone(),
            descriptor.output_kind.clone(),
            descriptor.phases.clone(),
        );
        if !dispatched_descriptors.insert(descriptor_key) {
            continue;
        }
        let input = if (*phase == PipelinePhase::TypeChecking
            && (type_prefix
                || registration_prefix
                || overload_prefix
                || elaboration_prefix
                || preparation_prefix
                || vc_group))
            || ((registration_prefix
                || overload_prefix
                || elaboration_prefix
                || preparation_prefix
                || vc_group)
                && *phase == PipelinePhase::RegistrationResolution)
            || ((overload_prefix || elaboration_prefix || preparation_prefix || vc_group)
                && *phase == PipelinePhase::OverloadResolution)
            || ((elaboration_prefix || preparation_prefix || vc_group)
                && *phase == PipelinePhase::Elaboration)
            || ((preparation_prefix || vc_group) && *phase == PipelinePhase::AlgorithmPreparation)
        {
            let bundle = (|| {
                use mizar_ir::{
                    dispatch_input::{PhaseDispatchInputBundle, SealedParentOutputHandle},
                    identity::{OutputKind, PipelinePhase as IrPhase, WorkUnit as IrWorkUnit},
                    storage::SchemaVersion,
                };
                let results = phase_results.get(&task.task.id)?;
                let expected: &[(&str, &str)] = if *phase == PipelinePhase::TypeChecking {
                    &[("SignatureCollection", "SourceTheoremModuleSummary")]
                } else if *phase == PipelinePhase::RegistrationResolution {
                    &[
                        ("SignatureCollection", "SourceTheoremModuleSummary"),
                        ("TypeChecking", "SourceTheoremTypedAst"),
                    ]
                } else if *phase == PipelinePhase::OverloadResolution {
                    &[
                        ("SignatureCollection", "SourceTheoremModuleSummary"),
                        ("TypeChecking", "SourceTheoremTypedAst"),
                        ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                    ]
                } else if *phase == PipelinePhase::Elaboration {
                    &[
                        ("SignatureCollection", "SourceTheoremModuleSummary"),
                        ("TypeChecking", "SourceTheoremTypedAst"),
                        ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                        ("OverloadResolution", "SourceTheoremMetadata"),
                    ]
                } else {
                    &[
                        ("SignatureCollection", "SourceTheoremModuleSummary"),
                        ("TypeChecking", "SourceTheoremTypedAst"),
                        ("RegistrationResolution", "SourceTheoremRegistrationClosure"),
                        ("OverloadResolution", "SourceTheoremMetadata"),
                        ("Elaboration", "SourceTheoremCore"),
                    ]
                };
                if results.len() != expected.len() {
                    return None;
                }
                let mizar_build::task_graph::WorkUnit::Module { module } = &task.task.unit else {
                    return None;
                };
                let unit = IrWorkUnit::new(format!(
                    "{:?}:{:?}",
                    module.package.as_str(),
                    module.path.as_str()
                ));
                for (result, (phase, kind)) in results.iter().zip(expected) {
                    let [parent] = result.output_refs.as_slice() else {
                        return None;
                    };
                    if result.status != PhaseStatus::Complete
                        || !result.diagnostics.is_empty()
                        || parent.snapshot() != task.snapshot
                        || parent.work_unit() != &unit
                        || parent.phase() != &IrPhase::new(*phase)
                        || parent.output_kind() != &OutputKind::new(*kind)
                        || parent.schema_version() != SchemaVersion::new(1)
                    {
                        return None;
                    }
                }
                let parents = if *phase == PipelinePhase::OverloadResolution {
                    let [_, typed, registration] = results.as_slice() else {
                        return None;
                    };
                    let typed = &typed.output_refs[0];
                    let registration = &registration.output_refs[0];
                    if registration.lineage().parents != vec![typed.output()] {
                        return None;
                    }
                    vec![typed, registration]
                } else if *phase == PipelinePhase::Elaboration {
                    let [_, typed, registration, metadata] = results.as_slice() else {
                        return None;
                    };
                    let typed = &typed.output_refs[0];
                    let registration = &registration.output_refs[0];
                    let metadata = &metadata.output_refs[0];
                    if metadata.lineage().parents.len() != 2
                        || !metadata.lineage().parents.contains(&typed.output())
                        || !metadata.lineage().parents.contains(&registration.output())
                    {
                        return None;
                    }
                    vec![metadata]
                } else if *phase == PipelinePhase::AlgorithmPreparation {
                    let [_, _, _, metadata, core] = results.as_slice() else {
                        return None;
                    };
                    let core = &core.output_refs[0];
                    if core.lineage().parents != vec![metadata.output_refs[0].output()] {
                        return None;
                    }
                    vec![core]
                } else {
                    let [parent] = results.last()?.output_refs.as_slice() else {
                        return None;
                    };
                    vec![parent]
                };
                let publisher = publisher?;
                let parents = parents
                    .into_iter()
                    .map(|parent| {
                        if source_load.is_some_and(|inputs| {
                            inputs
                                .captured
                                .is_some_and(|capture| capture.native_sources().is_some())
                        }) {
                            SealedParentOutputHandle::from_published_output(
                                publisher,
                                task.snapshot,
                                parent.clone(),
                            )
                        } else {
                            SealedParentOutputHandle::from_current_output(
                                publisher,
                                task.snapshot,
                                parent.clone(),
                            )
                        }
                        .ok()
                    })
                    .collect::<Option<Vec<_>>>()?;
                PhaseDispatchInputBundle::new(
                    dispatch_input.snapshot(),
                    dispatch_input.identities().input_hash(),
                    dispatch_input.identities().dependency_hashes().to_vec(),
                    parents,
                )
                .ok()
            })();
            let Some(bundle) = bundle else {
                return SchedulerDispatchOutcome::blocked(vec![dispatch_diagnostic(
                    task.task,
                    "missing_phase_input_identities",
                    "owner-provided phase input bundle is unavailable",
                )]);
            };
            PhaseInput::new(task.task.unit.clone(), bundle)
        } else {
            PhaseInput::new(task.task.unit.clone(), dispatch_input.clone())
        };
        match registry.execute_phase_with_resources(
            *phase,
            input,
            PhaseExecutionResources {
                cancellation: task.cancellation.clone(),
                output_publisher: publisher.cloned(),
                source_load,
                diagnostics: matches!(phase, PipelinePhase::SourceLoad | PipelinePhase::Frontend)
                    .then(|| {
                        let (diagnostic_phase, producer) = if *phase == PipelinePhase::SourceLoad {
                            (
                                mizar_diagnostics::failure_record::PipelinePhase::SourceLoad,
                                "SourceLoad",
                            )
                        } else {
                            (
                                mizar_diagnostics::failure_record::PipelinePhase::Frontend,
                                "Frontend",
                            )
                        };
                        mizar_diagnostics::sink::DiagnosticSink::new(
                            mizar_diagnostics::sink::DiagnosticProducerScope::new(
                                diagnostic_phase,
                                task.snapshot,
                                producer,
                            ),
                        )
                    }),
            },
        ) {
            Ok(result) => {
                let mut result = result.result;
                let status = result.status;
                let valid_outputs = result.output_refs.iter().all(|output| {
                    publisher.is_some_and(|publisher| {
                        (if source_load.is_some_and(|inputs| {
                            inputs
                                .captured
                                .is_some_and(|capture| capture.native_sources().is_some())
                        }) {
                            publisher.validate_published_output(task.snapshot, output)
                        } else {
                            publisher.validate_current_output(task.snapshot, output)
                        })
                        .is_ok()
                            && publisher.storage().validate_handle(output).is_ok()
                    })
                });
                let valid_snapshot = publisher.is_none_or(|publisher| {
                    publisher.validate_current_snapshot(task.snapshot).is_ok()
                });
                if !valid_outputs
                    || !valid_snapshot
                    || result
                        .diagnostics
                        .iter()
                        .any(|batch| batch.scope().source_snapshot() != task.snapshot)
                {
                    return SchedulerDispatchOutcome::failed(vec![dispatch_diagnostic(
                        task.task,
                        "invalid_phase_result",
                        "phase result failed snapshot or sealed-output validation",
                    )]);
                }
                if status != PhaseStatus::Complete {
                    result.output_refs.clear();
                }
                if result.artifact_commit.is_some() {
                    let eligible = (|| {
                        use crate::frontend_adapter::SourceTheoremArtifactCommitService;
                        use mizar_build::task_graph::{TaskKind, WorkUnit};
                        use mizar_ir::{
                            identity::{
                                OutputKind, PipelinePhase as IrPhase, WorkUnit as IrWorkUnit,
                            },
                            storage::SchemaVersion,
                        };
                        if *phase != PipelinePhase::ArtifactCommit
                            || task.task.kind != TaskKind::ArtifactCommit
                            || task.task.phases != [PipelinePhase::ArtifactCommit]
                            || status != PhaseStatus::Complete
                            || !result.diagnostics.is_empty()
                            || !registry.service_for_phase(*phase).is_ok_and(|service| {
                                std::any::Any::type_id(service)
                                    == std::any::TypeId::of::<SourceTheoremArtifactCommitService>()
                                    && &service.phase() == descriptor
                            })
                        {
                            return false;
                        }
                        let WorkUnit::Module { module } = &task.task.unit else {
                            return false;
                        };
                        let [output] = result.output_refs.as_slice() else {
                            return false;
                        };
                        let Some(publisher) = publisher else {
                            return false;
                        };
                        if !source_load.is_some_and(|inputs| {
                            inputs.snapshot.id == task.snapshot
                                && inputs.captured.is_some_and(|capture| {
                                    capture.native_sources().is_some()
                                        && capture.snapshot() == inputs.snapshot
                                })
                        }) {
                            return false;
                        }
                        let kind = OutputKind::new("SourceTheoremArtifactData");
                        output.snapshot() == task.snapshot
                            && output.phase() == &IrPhase::new("ArtifactCommit")
                            && output.work_unit()
                                == &IrWorkUnit::new(format!(
                                    "{:?}:{:?}",
                                    module.package.as_str(),
                                    module.path.as_str()
                                ))
                            && output.output_kind() == &kind
                            && output.schema_version() == SchemaVersion::new(1)
                            && publisher
                                .validate_current_output(task.snapshot, output)
                                .is_ok()
                    })();
                    if !eligible {
                        result.artifact_commit = None;
                    }
                }
                phase_results
                    .entry(task.task.id.clone())
                    .or_default()
                    .push(result);
                let diagnostic = phase_status_diagnostic(task.task, status);
                match status {
                    PhaseStatus::Complete => {}
                    PhaseStatus::Recoverable | PhaseStatus::Fatal => {
                        return SchedulerDispatchOutcome::failed(diagnostic.into_iter().collect());
                    }
                    PhaseStatus::Blocking => {
                        return SchedulerDispatchOutcome::blocked(diagnostic.into_iter().collect());
                    }
                    PhaseStatus::Cancelled => return SchedulerDispatchOutcome::cancelled(),
                }
            }
            Err(PhaseRegistryError::MissingPhaseService { .. }) => {
                return SchedulerDispatchOutcome::blocked(vec![dispatch_diagnostic(
                    task.task,
                    "missing_phase_service",
                    "phase service is unavailable at dispatch time",
                )]);
            }
            Err(error) => {
                return SchedulerDispatchOutcome::failed(vec![dispatch_diagnostic(
                    task.task,
                    "phase_registry_error",
                    error.to_string(),
                )]);
            }
        }
    }
    SchedulerDispatchOutcome::complete()
}

fn phase_status_diagnostic(
    task: &BuildTask,
    status: PhaseStatus,
) -> Option<SchedulerDiagnosticRef> {
    match status {
        PhaseStatus::Complete | PhaseStatus::Cancelled => None,
        PhaseStatus::Recoverable => Some(dispatch_diagnostic(
            task,
            "phase_recoverable",
            "phase completed with recoverable diagnostics",
        )),
        PhaseStatus::Blocking => Some(dispatch_diagnostic(
            task,
            "phase_blocking",
            "phase reported a blocking outcome",
        )),
        PhaseStatus::Fatal => Some(dispatch_diagnostic(
            task,
            "phase_fatal",
            "phase reported a fatal outcome",
        )),
    }
}

fn dispatch_diagnostic(
    task: &BuildTask,
    code: impl Into<String>,
    message: impl Into<String>,
) -> SchedulerDiagnosticRef {
    SchedulerDiagnosticRef::new(task.id.as_str(), code, message)
}
