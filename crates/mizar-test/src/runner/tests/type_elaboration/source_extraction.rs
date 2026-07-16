    #[test]
    fn source_reserve_extractor_preserves_builtin_declarations_and_rejects_gaps() {
        let builtin_source_id = source_id(91);
        let ast = reserve_ast(
            builtin_source_id,
            vec![
                reserve_item(vec!["x", "y"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["z"], ReserveTypeShape::Builtin("object")),
            ],
        );
        let module = ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));

        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let source_reserve =
            extract_builtin_source_reserve_declarations(&ast, module.clone(), &symbols)
                .expect("builtin reserve declarations should extract");

        assert_eq!(
            source_reserve
                .bindings()
                .iter()
                .map(|binding| (
                    binding.spelling.as_str(),
                    binding.type_spelling.as_str(),
                    &binding.type_head,
                    binding.type_range
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    "x",
                    "set",
                    &TypeHeadInput::BuiltinSet,
                    range(builtin_source_id, 18, 21),
                ),
                (
                    "y",
                    "set",
                    &TypeHeadInput::BuiltinSet,
                    range(builtin_source_id, 18, 21),
                ),
                (
                    "z",
                    "object",
                    &TypeHeadInput::BuiltinObject,
                    range(builtin_source_id, 38, 44),
                ),
            ]
        );

        let non_builtin = reserve_ast(
            source_id(92),
            vec![reserve_item(vec!["x"], ReserveTypeShape::NonBuiltin("T"))],
        );

        assert!(
            extract_builtin_source_reserve_declarations(&non_builtin, module.clone(), &symbols)
                .is_err(),
            "non-builtin type heads must stay on the external gap"
        );

        let attributed_symbols = source_symbol_env(module.clone());
        let unsupported = reserve_ast(
            source_id(93),
            vec![reserve_item(vec!["x"], ReserveTypeShape::AttributedSet)],
        );

        let attributed = extract_builtin_source_reserve_declarations(
            &unsupported,
            module.clone(),
            &attributed_symbols,
        )
        .expect("attribute-bearing builtin reserve type should extract");
        assert_eq!(attributed.bindings().len(), 1);
        assert_eq!(attributed.bindings()[0].type_attributes.len(), 1);
        assert_eq!(
            attributed.bindings()[0].type_attributes[0].polarity,
            AttributePolarity::Negative
        );

        let local_mode = reserve_ast(
            source_id(94),
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("Mode"),
            )],
        );
        let mode_symbols = source_mode_symbol_env(module.clone());
        let local_mode_reserve =
            extract_builtin_source_reserve_declarations(&local_mode, module.clone(), &mode_symbols)
                .expect("same-module mode reserve type should extract");
        assert_eq!(local_mode_reserve.bindings().len(), 1);
        assert!(matches!(
            local_mode_reserve.bindings()[0].type_head,
            TypeHeadInput::Symbol(_)
        ));

        let attributed_mode = reserve_ast(
            source_id(194),
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedQualifiedSymbol("Mode"),
            )],
        );
        let mode_attribute_symbols = source_mode_attribute_symbol_env(module.clone());
        let attributed_mode_reserve = extract_builtin_source_reserve_declarations(
            &attributed_mode,
            mode_attribute_symbols.module_id().clone(),
            &mode_attribute_symbols,
        )
        .expect("same-module attributed mode reserve type should extract");
        assert_eq!(attributed_mode_reserve.bindings().len(), 1);
        assert_eq!(
            attributed_mode_reserve.bindings()[0].type_attributes.len(),
            1
        );
        assert_eq!(
            attributed_mode_reserve.bindings()[0].type_attributes[0].polarity,
            AttributePolarity::Negative
        );
        assert!(matches!(
            attributed_mode_reserve.bindings()[0].type_head,
            TypeHeadInput::Symbol(_)
        ));

        let local_structure = reserve_ast(
            source_id(195),
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::QualifiedSymbol("Struct"),
            )],
        );
        let structure_symbols = source_structure_symbol_env(local_mode_reserve.module_id().clone());
        let local_structure_reserve = extract_builtin_source_reserve_declarations(
            &local_structure,
            structure_symbols.module_id().clone(),
            &structure_symbols,
        )
        .expect("same-module structure reserve type should extract");
        assert_eq!(local_structure_reserve.bindings().len(), 1);
        assert!(matches!(
            local_structure_reserve.bindings()[0].type_head,
            TypeHeadInput::Symbol(_)
        ));

        let attributed_structure = reserve_ast(
            source_id(196),
            vec![reserve_item(
                vec!["x"],
                ReserveTypeShape::AttributedQualifiedSymbol("Struct"),
            )],
        );
        let structure_attribute_symbols =
            source_structure_attribute_symbol_env(local_structure_reserve.module_id().clone());
        let attributed_structure_reserve = extract_builtin_source_reserve_declarations(
            &attributed_structure,
            structure_attribute_symbols.module_id().clone(),
            &structure_attribute_symbols,
        )
        .expect("same-module attributed structure reserve type should extract");
        assert_eq!(attributed_structure_reserve.bindings().len(), 1);
        assert_eq!(
            attributed_structure_reserve.bindings()[0]
                .type_attributes
                .len(),
            1
        );
        assert!(matches!(
            attributed_structure_reserve.bindings()[0].type_head,
            TypeHeadInput::Symbol(_)
        ));
    }

    #[test]
    fn source_reserve_extractor_preserves_local_mode_expansion_chain_payloads() {
        let source = source_id(199);
        let module = ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let symbols = source_mode_chain_symbol_env(module.clone());
        let ast = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::Builtin("set")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("A")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("B"),
            )],
        );

        let extraction =
            extract_builtin_source_reserve_declarations(&ast, module.clone(), &symbols)
                .expect("mode chain reserve should extract");
        let a = resolve_visible_type_head(&symbols, &module, "A").unwrap();
        let b = resolve_visible_type_head(&symbols, &module, "B").unwrap();
        assert_eq!(extraction.mode_expansions().len(), 2);
        assert!(matches!(
            extraction
                .mode_expansions()
                .get(&a)
                .map(|expansion| &expansion.radix.head),
            Some(TypeHeadInput::BuiltinSet)
        ));
        assert_eq!(
            extraction
                .mode_expansions()
                .get(&b)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(a.clone()))
        );

        let attributed_dependency = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::Builtin("set")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("A")),
            ],
            vec![
                reserve_item(vec!["z"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["y"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let attributed_extraction = extract_builtin_source_reserve_declarations(
            &attributed_dependency,
            module.clone(),
            &symbols,
        )
        .expect("attributed dependency reserve should still extract");
        assert!(
            attributed_extraction.mode_expansions().is_empty(),
            "any attributed reserve use in a local-mode chain withholds the chain expansion"
        );

        let attributed_root_symbols = source_mode_attribute_symbol_env(module.clone());
        let attributed_root = mode_chain_reserve_ast(
            source,
            [mode_definition("Mode", ReserveTypeShape::Builtin("set"))],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("Mode"),
            )],
        );
        let attributed_root_extraction = extract_builtin_source_reserve_declarations(
            &attributed_root,
            module.clone(),
            &attributed_root_symbols,
        )
        .expect("single attributed local-mode reserve should still extract");
        let attributed_root_mode =
            resolve_visible_type_head(&attributed_root_symbols, &module, "Mode").unwrap();
        assert_eq!(attributed_root_extraction.mode_expansions().len(), 1);
        assert!(matches!(
            attributed_root_extraction
                .mode_expansions()
                .get(&attributed_root_mode)
                .map(|expansion| &expansion.radix.head),
            Some(TypeHeadInput::BuiltinSet)
        ));

        let mixed_attributed_root = mode_chain_reserve_ast(
            source,
            [mode_definition("Mode", ReserveTypeShape::Builtin("set"))],
            vec![
                reserve_item(vec!["z"], ReserveTypeShape::QualifiedSymbol("Mode")),
                reserve_item(
                    vec!["y"],
                    ReserveTypeShape::AttributedQualifiedSymbol("Mode"),
                ),
            ],
        );
        let mixed_attributed_root_extraction = extract_builtin_source_reserve_declarations(
            &mixed_attributed_root,
            module.clone(),
            &attributed_root_symbols,
        )
        .expect("mixed local-mode reserve should still extract");
        assert!(
            mixed_attributed_root_extraction
                .mode_expansions()
                .is_empty(),
            "mixed bare/attributed uses of the same local mode still withhold expansion"
        );

        let structure_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let structure_rhs_symbols = source_local_symbols_env(
            structure_rhs_module.clone(),
            &[
                ("Struct", SymbolKind::Structure),
                ("Mode", SymbolKind::Mode),
            ],
        );
        let structure_rhs = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [mode_definition(
                "Mode",
                ReserveTypeShape::QualifiedSymbol("Struct"),
            )],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("Mode"),
            )],
        );
        let structure_rhs_extraction = extract_builtin_source_reserve_declarations(
            &structure_rhs,
            structure_rhs_module.clone(),
            &structure_rhs_symbols,
        )
        .expect("mode with structure RHS reserve should extract");
        let structure_rhs_mode =
            resolve_visible_type_head(&structure_rhs_symbols, &structure_rhs_module, "Mode")
                .unwrap();
        let structure_rhs_struct =
            resolve_visible_type_head(&structure_rhs_symbols, &structure_rhs_module, "Struct")
                .unwrap();
        assert_eq!(structure_rhs_extraction.mode_expansions().len(), 1);
        assert_eq!(
            structure_rhs_extraction
                .mode_expansions()
                .get(&structure_rhs_mode)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(structure_rhs_struct))
        );

        let attributed_structure_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let attributed_structure_rhs_symbols = source_local_symbols_env(
            attributed_structure_rhs_module.clone(),
            &[
                ("empty", SymbolKind::Attribute),
                ("Struct", SymbolKind::Structure),
                ("Mode", SymbolKind::Mode),
            ],
        );
        let attributed_structure_rhs = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [mode_definition(
                "Mode",
                ReserveTypeShape::QualifiedSymbol("Struct"),
            )],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("Mode"),
            )],
        );
        let attributed_structure_rhs_extraction = extract_builtin_source_reserve_declarations(
            &attributed_structure_rhs,
            attributed_structure_rhs_module.clone(),
            &attributed_structure_rhs_symbols,
        )
        .expect("attributed mode with structure RHS reserve should extract");
        let attributed_structure_rhs_mode = resolve_visible_type_head(
            &attributed_structure_rhs_symbols,
            &attributed_structure_rhs_module,
            "Mode",
        )
        .unwrap();
        let attributed_structure_rhs_struct = resolve_visible_type_head(
            &attributed_structure_rhs_symbols,
            &attributed_structure_rhs_module,
            "Struct",
        )
        .unwrap();
        assert_eq!(
            attributed_structure_rhs_extraction.mode_expansions().len(),
            1
        );
        assert_eq!(
            attributed_structure_rhs_extraction
                .mode_expansions()
                .get(&attributed_structure_rhs_mode)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(attributed_structure_rhs_struct))
        );

        let mixed_attributed_structure_rhs = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [mode_definition(
                "Mode",
                ReserveTypeShape::QualifiedSymbol("Struct"),
            )],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol("Mode")),
                reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("Mode"),
                ),
            ],
        );
        let mixed_attributed_structure_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &mixed_attributed_structure_rhs,
                attributed_structure_rhs_module.clone(),
                &attributed_structure_rhs_symbols,
            )
            .expect("mixed attributed structure-RHS reserve should still extract");
        assert!(
            mixed_attributed_structure_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "mixed bare/attributed uses still withhold direct structure-RHS expansion"
        );

        let attributed_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let attributed_rhs_symbols =
            source_mode_attribute_symbol_env(attributed_rhs_module.clone());
        let attributed_rhs = mode_chain_reserve_ast(
            source,
            [mode_definition("Mode", ReserveTypeShape::AttributedSet)],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("Mode"),
            )],
        );
        let attributed_rhs_extraction = extract_builtin_source_reserve_declarations(
            &attributed_rhs,
            attributed_rhs_module.clone(),
            &attributed_rhs_symbols,
        )
        .expect("mode with attributed builtin RHS reserve should extract");
        let attributed_rhs_mode =
            resolve_visible_type_head(&attributed_rhs_symbols, &attributed_rhs_module, "Mode")
                .unwrap();
        assert_eq!(attributed_rhs_extraction.mode_expansions().len(), 1);
        let attributed_rhs_expansion = attributed_rhs_extraction
            .mode_expansions()
            .get(&attributed_rhs_mode)
            .expect("attributed RHS mode expansion should be present");
        assert!(matches!(
            attributed_rhs_expansion.radix.head,
            TypeHeadInput::BuiltinSet
        ));
        assert_eq!(attributed_rhs_expansion.attributes.len(), 1);
        assert_eq!(
            attributed_rhs_expansion.attributes[0].polarity,
            AttributePolarity::Negative
        );

        let attributed_root_attributed_rhs = mode_chain_reserve_ast(
            source,
            [mode_definition("Mode", ReserveTypeShape::AttributedSet)],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("Mode"),
            )],
        );
        let attributed_root_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_root_attributed_rhs,
                attributed_rhs_module.clone(),
                &attributed_rhs_symbols,
            )
            .expect("attributed mode with attributed builtin RHS reserve should extract");
        assert_eq!(
            attributed_root_attributed_rhs_extraction.bindings()[0]
                .type_attributes
                .len(),
            1
        );
        assert_eq!(
            attributed_root_attributed_rhs_extraction
                .mode_expansions()
                .len(),
            1
        );
        let attributed_root_attributed_rhs_expansion = attributed_root_attributed_rhs_extraction
            .mode_expansions()
            .get(&attributed_rhs_mode)
            .expect("attributed-root attributed RHS expansion should be present");
        assert!(matches!(
            attributed_root_attributed_rhs_expansion.radix.head,
            TypeHeadInput::BuiltinSet
        ));
        assert_eq!(attributed_root_attributed_rhs_expansion.attributes.len(), 1);

        let attributed_object_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let attributed_object_rhs_symbols = source_local_symbols_env(
            attributed_object_rhs_module.clone(),
            &[
                ("empty", SymbolKind::Attribute),
                ("ObjectMode", SymbolKind::Mode),
            ],
        );
        let attributed_root_attributed_object_rhs = mode_chain_reserve_ast(
            source,
            [mode_definition(
                "ObjectMode",
                ReserveTypeShape::AttributedObject,
            )],
            vec![reserve_item(
                vec!["w"],
                ReserveTypeShape::AttributedQualifiedSymbol("ObjectMode"),
            )],
        );
        let attributed_root_attributed_object_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_root_attributed_object_rhs,
                attributed_object_rhs_module.clone(),
                &attributed_object_rhs_symbols,
            )
            .expect("attributed mode with attributed object RHS reserve should extract");
        let attributed_object_rhs_mode = resolve_visible_type_head(
            &attributed_object_rhs_symbols,
            &attributed_object_rhs_module,
            "ObjectMode",
        )
        .unwrap();
        let attributed_root_attributed_object_rhs_expansion =
            attributed_root_attributed_object_rhs_extraction
                .mode_expansions()
                .get(&attributed_object_rhs_mode)
                .expect("attributed-root attributed object RHS expansion should be present");
        assert!(matches!(
            attributed_root_attributed_object_rhs_expansion.radix.head,
            TypeHeadInput::BuiltinObject
        ));
        assert_eq!(
            attributed_root_attributed_object_rhs_expansion
                .attributes
                .len(),
            1
        );

        let mixed_attributed_root_attributed_rhs = mode_chain_reserve_ast(
            source,
            [mode_definition("Mode", ReserveTypeShape::AttributedSet)],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol("Mode")),
                reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("Mode"),
                ),
            ],
        );
        let mixed_attributed_root_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &mixed_attributed_root_attributed_rhs,
                attributed_rhs_module.clone(),
                &attributed_rhs_symbols,
            )
            .expect("mixed attributed-root attributed RHS reserve should still extract");
        assert!(
            mixed_attributed_root_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "mixed bare/attributed uses still withhold direct attributed-RHS expansion"
        );

        let attributed_chain_attributed_rhs_symbols = source_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
        );
        let attributed_chain_attributed_rhs_module =
            attributed_chain_attributed_rhs_symbols.module_id().clone();
        let attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("attributed chain ending in attributed RHS should still extract");
        let attributed_chain_attributed_b = resolve_visible_type_head(
            &attributed_chain_attributed_rhs_symbols,
            &attributed_chain_attributed_rhs_module,
            "B",
        )
        .unwrap();
        let attributed_chain_attributed_a = resolve_visible_type_head(
            &attributed_chain_attributed_rhs_symbols,
            &attributed_chain_attributed_rhs_module,
            "A",
        )
        .unwrap();
        assert_eq!(
            attributed_chain_attributed_rhs_extraction.bindings()[0]
                .type_attributes
                .len(),
            1
        );
        assert_eq!(
            attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .len(),
            2
        );
        let attributed_chain_attributed_b_expansion = attributed_chain_attributed_rhs_extraction
            .mode_expansions()
            .get(&attributed_chain_attributed_b)
            .expect("terminal attributed RHS expansion should be present");
        assert!(matches!(
            attributed_chain_attributed_b_expansion.radix.head,
            TypeHeadInput::BuiltinSet
        ));
        assert_eq!(attributed_chain_attributed_b_expansion.attributes.len(), 1);
        assert_eq!(
            attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .get(&attributed_chain_attributed_a)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(
                attributed_chain_attributed_b.clone()
            ))
        );

        let attributed_chain_attributed_object_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedObject),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let attributed_chain_attributed_object_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_chain_attributed_object_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("attributed chain ending in attributed object RHS should still extract");
        let attributed_chain_attributed_object_b_expansion =
            attributed_chain_attributed_object_rhs_extraction
                .mode_expansions()
                .get(&attributed_chain_attributed_b)
                .expect("terminal attributed object RHS expansion should be present");
        assert!(matches!(
            attributed_chain_attributed_object_b_expansion.radix.head,
            TypeHeadInput::BuiltinObject
        ));
        assert_eq!(
            attributed_chain_attributed_object_b_expansion
                .attributes
                .len(),
            1
        );
        assert!(
            attributed_chain_attributed_object_rhs_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_attributed_a)
        );

        let mixed_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("A")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let mixed_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &mixed_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("mixed attributed-RHS chain reserve should still extract");
        assert!(
            mixed_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds mixed bare/attributed uses of the attributed root"
        );

        let attributed_dependency_for_attributed_rhs_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::AttributedQualifiedSymbol("B")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let attributed_dependency_for_attributed_rhs_chain_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_dependency_for_attributed_rhs_chain,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("attributed dependency attributed-RHS chain reserve should still extract");
        assert!(
            !attributed_dependency_for_attributed_rhs_chain_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_attributed_a),
            "Task66 withholds attributed dependency modes"
        );

        let deeper_attributed_chain_attributed_rhs_symbols = source_local_symbols_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            &[
                ("empty", SymbolKind::Attribute),
                ("B", SymbolKind::Mode),
                ("A", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
            ],
        );
        let deeper_attributed_chain_attributed_rhs_module =
            deeper_attributed_chain_attributed_rhs_symbols
                .module_id()
                .clone();
        let deeper_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("A")),
            ],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("A")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("C")),
            ],
        );
        let deeper_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &deeper_attributed_chain_attributed_rhs,
                deeper_attributed_chain_attributed_rhs_module.clone(),
                &deeper_attributed_chain_attributed_rhs_symbols,
            )
            .expect("deeper attributed-RHS attributed chain reserve should still extract");
        let deeper_attributed_chain_attributed_c = resolve_visible_type_head(
            &deeper_attributed_chain_attributed_rhs_symbols,
            &deeper_attributed_chain_attributed_rhs_module,
            "C",
        )
        .unwrap();
        assert!(
            !deeper_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .contains_key(&deeper_attributed_chain_attributed_c),
            "Task66 remains capped at one dependency edge"
        );

        let duplicate_root_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let duplicate_root_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &duplicate_root_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("duplicate-root attributed RHS chain reserve should still extract");
        assert!(
            duplicate_root_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 requires a unique root mode definition"
        );

        let duplicate_terminal_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let duplicate_terminal_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &duplicate_terminal_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("duplicate-terminal attributed RHS chain reserve should still extract");
        assert!(
            duplicate_terminal_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 requires a unique terminal mode definition"
        );

        let forward_terminal_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("B", ReserveTypeShape::AttributedSet),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let forward_terminal_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &forward_terminal_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("forward terminal attributed RHS chain reserve should still extract");
        assert!(
            forward_terminal_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 requires source order B -> A -> reserve"
        );

        let attribute_args_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSetWithAttributeArgs),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let attribute_args_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &attribute_args_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("attribute-argument attributed RHS chain reserve should still extract");
        assert!(
            attribute_args_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds terminal attributed RHS attribute arguments"
        );

        let argument_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbolWithArgs("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let argument_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &argument_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("argument-bearing attributed RHS chain reserve should still extract");
        assert!(
            argument_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds argument-bearing dependency RHSs"
        );

        let contextual_root_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                contextual_mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let contextual_root_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &contextual_root_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("contextual root attributed RHS chain reserve should still extract");
        assert!(
            contextual_root_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds contextual root definitions"
        );

        let contextual_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                contextual_mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let contextual_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &contextual_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("contextual attributed RHS chain reserve should still extract");
        assert!(
            contextual_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds contextual dependency definitions"
        );

        let parameterized_root_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                parameterized_mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let parameterized_root_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &parameterized_root_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("parameterized root attributed RHS chain reserve should still extract");
        assert!(
            parameterized_root_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds parameterized root definitions"
        );

        let parameterized_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                parameterized_mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let parameterized_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &parameterized_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("parameterized attributed RHS chain reserve should still extract");
        assert!(
            parameterized_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds parameterized dependency definitions"
        );

        let recovered_root_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                recovered_mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let recovered_root_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &recovered_root_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("recovered root attributed RHS chain reserve should still extract");
        assert!(
            recovered_root_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds recovered root definitions"
        );

        let recovered_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                recovered_mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let recovered_attributed_chain_attributed_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &recovered_attributed_chain_attributed_rhs,
                attributed_chain_attributed_rhs_module.clone(),
                &attributed_chain_attributed_rhs_symbols,
            )
            .expect("recovered attributed RHS chain reserve should still extract");
        assert!(
            recovered_attributed_chain_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds recovered dependency definitions"
        );

        let imported_attribute_attributed_chain_attributed_rhs_symbols =
            imported_attribute_mode_chain_symbol_env(ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("bridge"),
            ));
        let imported_attribute_attributed_chain_attributed_rhs_module =
            imported_attribute_attributed_chain_attributed_rhs_symbols
                .module_id()
                .clone();
        let imported_attribute_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &imported_attribute_attributed_chain_attributed_rhs,
                imported_attribute_attributed_chain_attributed_rhs_module,
                &imported_attribute_attributed_chain_attributed_rhs_symbols,
            )
            .is_err(),
            "Task66 withholds imported reserve-head or terminal RHS attributes"
        );

        let ambiguous_attribute_attributed_chain_attributed_rhs_symbols =
            ambiguous_attribute_mode_chain_symbol_env(ResolverModuleId::new(
                PackageId::new("test"),
                ModulePath::new("bridge"),
            ));
        let ambiguous_attribute_attributed_chain_attributed_rhs_module =
            ambiguous_attribute_attributed_chain_attributed_rhs_symbols
                .module_id()
                .clone();
        let ambiguous_attribute_attributed_chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &ambiguous_attribute_attributed_chain_attributed_rhs,
                ambiguous_attribute_attributed_chain_attributed_rhs_module,
                &ambiguous_attribute_attributed_chain_attributed_rhs_symbols,
            )
            .is_err(),
            "Task66 withholds ambiguous reserve-head or terminal RHS attributes"
        );

        let imported_attributed_chain_attributed_rhs_root_symbols = imported_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            "A",
        );
        let imported_attributed_chain_attributed_rhs_root_module =
            imported_attributed_chain_attributed_rhs_root_symbols
                .module_id()
                .clone();
        let imported_attributed_chain_attributed_rhs_root = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &imported_attributed_chain_attributed_rhs_root,
                imported_attributed_chain_attributed_rhs_root_module,
                &imported_attributed_chain_attributed_rhs_root_symbols,
            )
            .is_err(),
            "Task66 withholds imported attributed roots"
        );

        let imported_attributed_chain_attributed_rhs_dependency_symbols =
            imported_mode_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "B",
            );
        let imported_attributed_chain_attributed_rhs_dependency_module =
            imported_attributed_chain_attributed_rhs_dependency_symbols
                .module_id()
                .clone();
        let imported_attributed_chain_attributed_rhs_dependency = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let imported_attributed_chain_attributed_rhs_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &imported_attributed_chain_attributed_rhs_dependency,
                imported_attributed_chain_attributed_rhs_dependency_module,
                &imported_attributed_chain_attributed_rhs_dependency_symbols,
            )
            .expect("imported dependency attributed RHS chain reserve should still extract");
        assert!(
            imported_attributed_chain_attributed_rhs_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds imported dependency modes"
        );

        let ambiguous_attributed_chain_attributed_rhs_root_symbols =
            ambiguous_attributed_mode_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "A",
            );
        let ambiguous_attributed_chain_attributed_rhs_root_module =
            ambiguous_attributed_chain_attributed_rhs_root_symbols
                .module_id()
                .clone();
        let ambiguous_attributed_chain_attributed_rhs_root = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &ambiguous_attributed_chain_attributed_rhs_root,
                ambiguous_attributed_chain_attributed_rhs_root_module,
                &ambiguous_attributed_chain_attributed_rhs_root_symbols,
            )
            .is_err(),
            "Task66 withholds ambiguous attributed roots"
        );

        let ambiguous_attributed_chain_attributed_rhs_dependency_symbols =
            ambiguous_attributed_mode_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "B",
            );
        let ambiguous_attributed_chain_attributed_rhs_dependency_module =
            ambiguous_attributed_chain_attributed_rhs_dependency_symbols
                .module_id()
                .clone();
        let ambiguous_attributed_chain_attributed_rhs_dependency = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let ambiguous_attributed_chain_attributed_rhs_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &ambiguous_attributed_chain_attributed_rhs_dependency,
                ambiguous_attributed_chain_attributed_rhs_dependency_module,
                &ambiguous_attributed_chain_attributed_rhs_dependency_symbols,
            )
            .expect("ambiguous dependency attributed RHS chain reserve should still extract");
        assert!(
            ambiguous_attributed_chain_attributed_rhs_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task66 withholds ambiguous dependency modes"
        );

        let chain_structure_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let chain_structure_rhs_symbols = source_local_symbols_env(
            chain_structure_rhs_module.clone(),
            &[
                ("Struct", SymbolKind::Structure),
                ("B", SymbolKind::Mode),
                ("A", SymbolKind::Mode),
            ],
        );
        let chain_structure_rhs = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let chain_structure_rhs_extraction = extract_builtin_source_reserve_declarations(
            &chain_structure_rhs,
            chain_structure_rhs_module.clone(),
            &chain_structure_rhs_symbols,
        )
        .expect("mode chain ending in structure RHS reserve should still extract");
        let chain_structure_b = resolve_visible_type_head(
            &chain_structure_rhs_symbols,
            &chain_structure_rhs_module,
            "B",
        )
        .unwrap();
        let chain_structure_a = resolve_visible_type_head(
            &chain_structure_rhs_symbols,
            &chain_structure_rhs_module,
            "A",
        )
        .unwrap();
        let chain_structure_struct = resolve_visible_type_head(
            &chain_structure_rhs_symbols,
            &chain_structure_rhs_module,
            "Struct",
        )
        .unwrap();
        assert_eq!(chain_structure_rhs_extraction.mode_expansions().len(), 2);
        assert_eq!(
            chain_structure_rhs_extraction
                .mode_expansions()
                .get(&chain_structure_b)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(chain_structure_struct))
        );
        assert_eq!(
            chain_structure_rhs_extraction
                .mode_expansions()
                .get(&chain_structure_a)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(chain_structure_b))
        );

        let attributed_chain_structure_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let attributed_chain_structure_rhs_symbols = source_local_symbols_env(
            attributed_chain_structure_rhs_module.clone(),
            &[
                ("empty", SymbolKind::Attribute),
                ("Struct", SymbolKind::Structure),
                ("A", SymbolKind::Mode),
                ("B", SymbolKind::Mode),
            ],
        );
        let attributed_chain_structure_rhs = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let attributed_chain_structure_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_chain_structure_rhs,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("attributed mode chain ending in structure RHS should still extract");
        let attributed_chain_structure_b = resolve_visible_type_head(
            &attributed_chain_structure_rhs_symbols,
            &attributed_chain_structure_rhs_module,
            "B",
        )
        .unwrap();
        let attributed_chain_structure_a = resolve_visible_type_head(
            &attributed_chain_structure_rhs_symbols,
            &attributed_chain_structure_rhs_module,
            "A",
        )
        .unwrap();
        let attributed_chain_structure_struct = resolve_visible_type_head(
            &attributed_chain_structure_rhs_symbols,
            &attributed_chain_structure_rhs_module,
            "Struct",
        )
        .unwrap();
        assert_eq!(
            attributed_chain_structure_rhs_extraction.bindings()[0]
                .type_attributes
                .len(),
            1
        );
        assert_eq!(
            attributed_chain_structure_rhs_extraction
                .mode_expansions()
                .len(),
            2
        );
        assert_eq!(
            attributed_chain_structure_rhs_extraction
                .mode_expansions()
                .get(&attributed_chain_structure_b)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(attributed_chain_structure_struct))
        );
        assert_eq!(
            attributed_chain_structure_rhs_extraction
                .mode_expansions()
                .get(&attributed_chain_structure_a)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(attributed_chain_structure_b.clone()))
        );

        let cached_attributed_chain_structure_rhs = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let cached_attributed_chain_structure_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &cached_attributed_chain_structure_rhs,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("cached attributed structure-RHS chain reserve should still extract");
        assert!(
            cached_attributed_chain_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_structure_b)
        );
        assert!(
            cached_attributed_chain_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_structure_a),
            "cached direct structure-RHS payload may feed the attributed one-edge chain"
        );

        let mixed_attributed_chain_structure_rhs = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("A")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let mixed_attributed_chain_structure_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &mixed_attributed_chain_structure_rhs,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("mixed attributed structure-RHS chain reserve should still extract");
        assert!(
            mixed_attributed_chain_structure_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "mixed bare/attributed uses of the attributed structure-chain root withhold expansion"
        );

        let attributed_dependency_for_structure_chain = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::AttributedQualifiedSymbol("B")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let attributed_dependency_for_structure_chain_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_dependency_for_structure_chain,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("attributed dependency structure-RHS chain reserve should still extract");
        assert!(
            !attributed_dependency_for_structure_chain_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_structure_a),
            "an attributed dependency mode stays outside the attributed-root structure chain"
        );

        let deeper_attributed_chain_structure_symbols = source_local_symbols_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            &[
                ("empty", SymbolKind::Attribute),
                ("Struct", SymbolKind::Structure),
                ("B", SymbolKind::Mode),
                ("A", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
            ],
        );
        let deeper_attributed_chain_structure_module = deeper_attributed_chain_structure_symbols
            .module_id()
            .clone();
        let deeper_attributed_chain_structure_rhs = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("A")),
            ],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("A")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("C")),
            ],
        );
        let deeper_attributed_chain_structure_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &deeper_attributed_chain_structure_rhs,
                deeper_attributed_chain_structure_module.clone(),
                &deeper_attributed_chain_structure_symbols,
            )
            .expect("deeper attributed structure-RHS chain reserve should still extract");
        let deeper_attributed_chain_structure_c = resolve_visible_type_head(
            &deeper_attributed_chain_structure_symbols,
            &deeper_attributed_chain_structure_module,
            "C",
        )
        .unwrap();
        assert!(
            !deeper_attributed_chain_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&deeper_attributed_chain_structure_c),
            "attributed local-mode structure chains remain capped at one dependency edge"
        );

        let duplicate_attributed_chain_structure_dependency =
            mode_chain_reserve_ast_with_structures(
                source,
                ["Struct"],
                [
                    mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                ],
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("A"),
                )],
            );
        let duplicate_attributed_chain_structure_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &duplicate_attributed_chain_structure_dependency,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("duplicate dependency structure-RHS chain reserve should still extract");
        assert!(
            duplicate_attributed_chain_structure_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 requires a unique dependency mode definition"
        );

        let duplicate_attributed_chain_structure_root = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let duplicate_attributed_chain_structure_root_extraction =
            extract_builtin_source_reserve_declarations(
                &duplicate_attributed_chain_structure_root,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("duplicate root structure-RHS chain reserve should still extract");
        assert!(
            duplicate_attributed_chain_structure_root_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 requires a unique attributed root mode definition"
        );

        let duplicate_attributed_chain_structure_definition =
            mode_chain_reserve_ast_with_structures(
                source,
                ["Struct", "Struct"],
                [
                    mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                ],
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("A"),
                )],
            );
        let duplicate_attributed_chain_structure_definition_extraction =
            extract_builtin_source_reserve_declarations(
                &duplicate_attributed_chain_structure_definition,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("duplicate structure definition reserve should still extract");
        assert!(
            duplicate_attributed_chain_structure_definition_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 requires a unique terminal structure definition"
        );

        let forward_attributed_chain_structure_dependency = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let forward_attributed_chain_structure_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &forward_attributed_chain_structure_dependency,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("forward dependency structure-RHS chain reserve should still extract");
        assert!(
            forward_attributed_chain_structure_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 requires the dependency definition to precede the attributed root definition"
        );

        let forward_attributed_chain_structure_terminal =
            mode_chain_reserve_ast_with_trailing_structures(
                source,
                [
                    mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                ],
                ["Struct"],
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("A"),
                )],
            );
        let forward_attributed_chain_structure_terminal_extraction =
            extract_builtin_source_reserve_declarations(
                &forward_attributed_chain_structure_terminal,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("forward terminal structure reserve should still extract");
        assert!(
            forward_attributed_chain_structure_terminal_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 requires the structure definition to precede the terminal mode definition"
        );

        let contextual_attributed_chain_structure_dependency =
            mode_chain_reserve_ast_with_structures(
                source,
                ["Struct"],
                [
                    contextual_mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                ],
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("A"),
                )],
            );
        let contextual_attributed_chain_structure_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &contextual_attributed_chain_structure_dependency,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("contextual dependency structure-RHS chain reserve should still extract");
        assert!(
            contextual_attributed_chain_structure_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds contextual dependency mode definitions"
        );

        let contextual_attributed_chain_structure_root = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                contextual_mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let contextual_attributed_chain_structure_root_extraction =
            extract_builtin_source_reserve_declarations(
                &contextual_attributed_chain_structure_root,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("contextual root structure-RHS chain reserve should still extract");
        assert!(
            contextual_attributed_chain_structure_root_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds contextual attributed root mode definitions"
        );

        let parameterized_attributed_chain_structure_dependency =
            mode_chain_reserve_ast_with_structures(
                source,
                ["Struct"],
                [
                    parameterized_mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                ],
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("A"),
                )],
            );
        let parameterized_attributed_chain_structure_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &parameterized_attributed_chain_structure_dependency,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("parameterized dependency structure-RHS chain reserve should still extract");
        assert!(
            parameterized_attributed_chain_structure_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds parameterized dependency mode definitions"
        );

        let recovered_attributed_chain_structure_dependency =
            mode_chain_reserve_ast_with_structures(
                source,
                ["Struct"],
                [
                    recovered_mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                ],
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("A"),
                )],
            );
        let recovered_attributed_chain_structure_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &recovered_attributed_chain_structure_dependency,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("recovered dependency structure-RHS chain reserve should still extract");
        assert!(
            recovered_attributed_chain_structure_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds recovered dependency mode definitions"
        );

        let argument_attributed_chain_structure_terminal = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbolWithArgs("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let argument_attributed_chain_structure_terminal_extraction =
            extract_builtin_source_reserve_declarations(
                &argument_attributed_chain_structure_terminal,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("argument-bearing terminal structure reserve should still extract");
        assert!(
            argument_attributed_chain_structure_terminal_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds argument-bearing terminal structure RHSs"
        );

        let argument_attributed_chain_structure_dependency = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbolWithArgs("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let argument_attributed_chain_structure_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &argument_attributed_chain_structure_dependency,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .expect("argument-bearing dependency reserve should still extract");
        assert!(
            !argument_attributed_chain_structure_dependency_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_structure_a),
            "Task65 withholds argument-bearing dependency RHSs"
        );

        let reserve_attribute_args_attributed_chain_structure =
            mode_chain_reserve_ast_with_structures(
                source,
                ["Struct"],
                [
                    mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                ],
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbolWithAttributeArgs("A"),
                )],
            );
        assert!(
            extract_builtin_source_reserve_declarations(
                &reserve_attribute_args_attributed_chain_structure,
                attributed_chain_structure_rhs_module.clone(),
                &attributed_chain_structure_rhs_symbols,
            )
            .is_err(),
            "Task65 withholds reserve-head attribute arguments"
        );

        let imported_attributed_chain_structure_root_symbols =
            imported_structure_mode_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "A",
            );
        let imported_attributed_chain_structure_root_module =
            imported_attributed_chain_structure_root_symbols
                .module_id()
                .clone();
        let imported_attributed_chain_structure_root = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &imported_attributed_chain_structure_root,
                imported_attributed_chain_structure_root_module,
                &imported_attributed_chain_structure_root_symbols,
            )
            .is_err(),
            "Task65 withholds imported attributed roots"
        );

        let imported_attributed_chain_structure_dependency_symbols =
            imported_structure_mode_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "B",
            );
        let imported_attributed_chain_structure_dependency_module =
            imported_attributed_chain_structure_dependency_symbols
                .module_id()
                .clone();
        let imported_attributed_chain_structure_dependency = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let imported_attributed_chain_structure_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &imported_attributed_chain_structure_dependency,
                imported_attributed_chain_structure_dependency_module,
                &imported_attributed_chain_structure_dependency_symbols,
            )
            .expect("imported dependency structure-RHS chain reserve should still extract");
        assert!(
            imported_attributed_chain_structure_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds imported dependency modes"
        );

        let imported_attributed_chain_structure_terminal_symbols =
            imported_structure_mode_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "Struct",
            );
        let imported_attributed_chain_structure_terminal_module =
            imported_attributed_chain_structure_terminal_symbols
                .module_id()
                .clone();
        let imported_attributed_chain_structure_terminal = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let imported_attributed_chain_structure_terminal_extraction =
            extract_builtin_source_reserve_declarations(
                &imported_attributed_chain_structure_terminal,
                imported_attributed_chain_structure_terminal_module,
                &imported_attributed_chain_structure_terminal_symbols,
            )
            .expect("imported terminal structure chain reserve should still extract");
        assert!(
            imported_attributed_chain_structure_terminal_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds imported terminal structures"
        );

        let ambiguous_attributed_chain_structure_root_symbols =
            ambiguous_attributed_structure_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "A",
            );
        let ambiguous_attributed_chain_structure_root_module =
            ambiguous_attributed_chain_structure_root_symbols
                .module_id()
                .clone();
        let ambiguous_attributed_chain_structure_root = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &ambiguous_attributed_chain_structure_root,
                ambiguous_attributed_chain_structure_root_module,
                &ambiguous_attributed_chain_structure_root_symbols,
            )
            .is_err(),
            "Task65 withholds ambiguous attributed roots"
        );

        let ambiguous_attributed_chain_structure_dependency_symbols =
            ambiguous_attributed_structure_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "B",
            );
        let ambiguous_attributed_chain_structure_dependency_module =
            ambiguous_attributed_chain_structure_dependency_symbols
                .module_id()
                .clone();
        let ambiguous_attributed_chain_structure_dependency =
            mode_chain_reserve_ast_with_structures(
                source,
                ["Struct"],
                [
                    mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                    mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                ],
                vec![reserve_item(
                    vec!["z"],
                    ReserveTypeShape::AttributedQualifiedSymbol("A"),
                )],
            );
        let ambiguous_attributed_chain_structure_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &ambiguous_attributed_chain_structure_dependency,
                ambiguous_attributed_chain_structure_dependency_module,
                &ambiguous_attributed_chain_structure_dependency_symbols,
            )
            .expect("ambiguous dependency structure-RHS chain reserve should still extract");
        assert!(
            ambiguous_attributed_chain_structure_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds ambiguous dependency modes"
        );

        let ambiguous_attributed_chain_structure_terminal_symbols =
            ambiguous_attributed_structure_chain_symbol_env(
                ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
                "Struct",
            );
        let ambiguous_attributed_chain_structure_terminal_module =
            ambiguous_attributed_chain_structure_terminal_symbols
                .module_id()
                .clone();
        let ambiguous_attributed_chain_structure_terminal = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let ambiguous_attributed_chain_structure_terminal_extraction =
            extract_builtin_source_reserve_declarations(
                &ambiguous_attributed_chain_structure_terminal,
                ambiguous_attributed_chain_structure_terminal_module,
                &ambiguous_attributed_chain_structure_terminal_symbols,
            )
            .expect("ambiguous terminal structure chain reserve should still extract");
        assert!(
            ambiguous_attributed_chain_structure_terminal_extraction
                .mode_expansions()
                .is_empty(),
            "Task65 withholds ambiguous terminal structures"
        );

        let attributed_chain_bare_rhs_symbols = source_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
        );
        let attributed_chain_bare_rhs_module =
            attributed_chain_bare_rhs_symbols.module_id().clone();
        let attributed_chain_bare_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let attributed_chain_bare_rhs_extraction = extract_builtin_source_reserve_declarations(
            &attributed_chain_bare_rhs,
            attributed_chain_bare_rhs_module.clone(),
            &attributed_chain_bare_rhs_symbols,
        )
        .expect("attributed mode chain ending in bare builtin RHS should extract");
        let attributed_chain_bare_b = resolve_visible_type_head(
            &attributed_chain_bare_rhs_symbols,
            &attributed_chain_bare_rhs_module,
            "B",
        )
        .unwrap();
        let attributed_chain_bare_a = resolve_visible_type_head(
            &attributed_chain_bare_rhs_symbols,
            &attributed_chain_bare_rhs_module,
            "A",
        )
        .unwrap();
        assert_eq!(
            attributed_chain_bare_rhs_extraction.bindings()[0]
                .type_attributes
                .len(),
            1
        );
        assert_eq!(
            attributed_chain_bare_rhs_extraction.mode_expansions().len(),
            2
        );
        assert!(matches!(
            attributed_chain_bare_rhs_extraction
                .mode_expansions()
                .get(&attributed_chain_bare_b)
                .map(|expansion| &expansion.radix.head),
            Some(TypeHeadInput::BuiltinSet)
        ));
        assert_eq!(
            attributed_chain_bare_rhs_extraction
                .mode_expansions()
                .get(&attributed_chain_bare_a)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(attributed_chain_bare_b.clone()))
        );

        let attributed_chain_object_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("object")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let attributed_chain_object_rhs_extraction = extract_builtin_source_reserve_declarations(
            &attributed_chain_object_rhs,
            attributed_chain_bare_rhs_module.clone(),
            &attributed_chain_bare_rhs_symbols,
        )
        .expect("attributed mode chain ending in object RHS should extract");
        assert!(matches!(
            attributed_chain_object_rhs_extraction
                .mode_expansions()
                .get(&attributed_chain_bare_b)
                .map(|expansion| &expansion.radix.head),
            Some(TypeHeadInput::BuiltinObject)
        ));
        assert!(
            attributed_chain_object_rhs_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_bare_a)
        );

        let cached_attributed_chain_bare_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let cached_attributed_chain_bare_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &cached_attributed_chain_bare_rhs,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("cached attributed bare-RHS chain reserve should still extract");
        assert!(
            cached_attributed_chain_bare_rhs_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_bare_b)
        );
        assert!(
            cached_attributed_chain_bare_rhs_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_bare_a),
            "cached bare-builtin terminal payload may feed the attributed one-edge chain"
        );

        let deeper_attributed_chain_bare_symbols = source_local_symbols_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            &[
                ("empty", SymbolKind::Attribute),
                ("B", SymbolKind::Mode),
                ("A", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
            ],
        );
        let deeper_attributed_chain_bare_module =
            deeper_attributed_chain_bare_symbols.module_id().clone();
        let deeper_attributed_chain_bare = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("A")),
            ],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("A")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("C")),
            ],
        );
        let deeper_attributed_chain_bare_extraction = extract_builtin_source_reserve_declarations(
            &deeper_attributed_chain_bare,
            deeper_attributed_chain_bare_module.clone(),
            &deeper_attributed_chain_bare_symbols,
        )
        .expect("deeper attributed bare-RHS chain reserve should still extract");
        let deeper_attributed_chain_c = resolve_visible_type_head(
            &deeper_attributed_chain_bare_symbols,
            &deeper_attributed_chain_bare_module,
            "C",
        )
        .unwrap();
        assert!(
            !deeper_attributed_chain_bare_extraction
                .mode_expansions()
                .contains_key(&deeper_attributed_chain_c),
            "attributed local-mode chains remain capped at one dependency edge"
        );

        let mixed_attributed_chain_bare_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("A")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let mixed_attributed_chain_bare_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &mixed_attributed_chain_bare_rhs,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("mixed attributed bare-RHS chain reserve should still extract");
        assert!(
            mixed_attributed_chain_bare_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "mixed bare/attributed uses of the attributed chain root withhold expansion"
        );

        let attributed_dependency_for_attributed_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::AttributedQualifiedSymbol("B")),
                reserve_item(vec!["z"], ReserveTypeShape::AttributedQualifiedSymbol("A")),
            ],
        );
        let attributed_dependency_for_attributed_chain_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_dependency_for_attributed_chain,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("attributed dependency chain reserve should still extract");
        assert!(
            !attributed_dependency_for_attributed_chain_extraction
                .mode_expansions()
                .contains_key(&attributed_chain_bare_a),
            "an attributed dependency mode stays outside the attributed-root chain slice"
        );

        let attributed_chain_with_mode_args = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbolWithArgs("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let attributed_chain_with_mode_args_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_chain_with_mode_args,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("argument-bearing attributed bare-RHS chain reserve should still extract");
        assert!(
            attributed_chain_with_mode_args_extraction
                .mode_expansions()
                .is_empty(),
            "argument-bearing mode dependencies remain outside Task64"
        );

        let contextual_attributed_chain_bare_rhs = mode_chain_reserve_ast(
            source,
            [
                contextual_mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let contextual_attributed_chain_bare_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &contextual_attributed_chain_bare_rhs,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("contextual attributed bare-RHS chain reserve should still extract");
        assert!(
            contextual_attributed_chain_bare_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "definition-local context keeps attributed bare-RHS chains withheld"
        );

        let parameterized_attributed_chain_bare_rhs = mode_chain_reserve_ast(
            source,
            [
                parameterized_mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let parameterized_attributed_chain_bare_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &parameterized_attributed_chain_bare_rhs,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("parameterized attributed bare-RHS chain reserve should still extract");
        assert!(
            parameterized_attributed_chain_bare_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "parameterized definitions remain outside Task64"
        );

        let recovered_attributed_chain_bare_rhs = mode_chain_reserve_ast(
            source,
            [
                recovered_mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let recovered_attributed_chain_bare_rhs_extraction =
            extract_builtin_source_reserve_declarations(
                &recovered_attributed_chain_bare_rhs,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("recovered attributed bare-RHS chain reserve should still extract");
        assert!(
            recovered_attributed_chain_bare_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "recovered definitions remain outside Task64"
        );

        let contextual_attributed_chain_head = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                contextual_mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let contextual_attributed_chain_head_extraction =
            extract_builtin_source_reserve_declarations(
                &contextual_attributed_chain_head,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("contextual attributed chain head reserve should still extract");
        assert!(
            contextual_attributed_chain_head_extraction
                .mode_expansions()
                .is_empty(),
            "definition-local context on the attributed root keeps Task64 withheld"
        );

        let parameterized_attributed_chain_head = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                parameterized_mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let parameterized_attributed_chain_head_extraction =
            extract_builtin_source_reserve_declarations(
                &parameterized_attributed_chain_head,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("parameterized attributed chain head reserve should still extract");
        assert!(
            parameterized_attributed_chain_head_extraction
                .mode_expansions()
                .is_empty(),
            "parameterized attributed root definitions remain outside Task64"
        );

        let recovered_attributed_chain_head = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                recovered_mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let recovered_attributed_chain_head_extraction =
            extract_builtin_source_reserve_declarations(
                &recovered_attributed_chain_head,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("recovered attributed chain head reserve should still extract");
        assert!(
            recovered_attributed_chain_head_extraction
                .mode_expansions()
                .is_empty(),
            "recovered attributed root definitions remain outside Task64"
        );

        let duplicate_attributed_chain_dependency = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("B", ReserveTypeShape::Builtin("object")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let duplicate_attributed_chain_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &duplicate_attributed_chain_dependency,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("duplicate dependency definition reserve should still extract");
        assert!(
            duplicate_attributed_chain_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task64 requires a unique dependency mode definition"
        );

        let duplicate_attributed_chain_head = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let duplicate_attributed_chain_head_extraction =
            extract_builtin_source_reserve_declarations(
                &duplicate_attributed_chain_head,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("duplicate attributed root definition reserve should still extract");
        assert!(
            duplicate_attributed_chain_head_extraction
                .mode_expansions()
                .is_empty(),
            "Task64 requires a unique attributed root mode definition"
        );

        let forward_attributed_chain_dependency = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("B", ReserveTypeShape::Builtin("set")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let forward_attributed_chain_dependency_extraction =
            extract_builtin_source_reserve_declarations(
                &forward_attributed_chain_dependency,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .expect("forward dependency attributed chain reserve should still extract");
        assert!(
            forward_attributed_chain_dependency_extraction
                .mode_expansions()
                .is_empty(),
            "Task64 requires the dependency definition to precede the attributed root definition"
        );

        let reserve_attribute_args_attributed_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbolWithAttributeArgs("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &reserve_attribute_args_attributed_chain,
                attributed_chain_bare_rhs_module.clone(),
                &attributed_chain_bare_rhs_symbols,
            )
            .is_err(),
            "reserve-head attribute arguments remain outside Task64"
        );

        let imported_root_symbols = imported_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            "A",
        );
        let imported_root_module = imported_root_symbols.module_id().clone();
        let imported_root_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &imported_root_chain,
                imported_root_module,
                &imported_root_symbols,
            )
            .is_err(),
            "imported attributed roots remain outside Task64"
        );

        let imported_dependency_symbols = imported_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            "B",
        );
        let imported_dependency_module = imported_dependency_symbols.module_id().clone();
        let imported_dependency_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let imported_dependency_extraction = extract_builtin_source_reserve_declarations(
            &imported_dependency_chain,
            imported_dependency_module,
            &imported_dependency_symbols,
        )
        .expect("imported dependency attributed chain reserve should still extract");
        assert!(
            imported_dependency_extraction.mode_expansions().is_empty(),
            "imported dependencies remain outside Task64"
        );

        let ambiguous_root_symbols = ambiguous_attributed_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            "A",
        );
        let ambiguous_root_module = ambiguous_root_symbols.module_id().clone();
        let ambiguous_root_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        assert!(
            extract_builtin_source_reserve_declarations(
                &ambiguous_root_chain,
                ambiguous_root_module,
                &ambiguous_root_symbols,
            )
            .is_err(),
            "ambiguous attributed roots remain outside Task64"
        );

        let ambiguous_dependency_symbols = ambiguous_attributed_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            "B",
        );
        let ambiguous_dependency_module = ambiguous_dependency_symbols.module_id().clone();
        let ambiguous_dependency_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::Builtin("set")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::AttributedQualifiedSymbol("A"),
            )],
        );
        let ambiguous_dependency_extraction = extract_builtin_source_reserve_declarations(
            &ambiguous_dependency_chain,
            ambiguous_dependency_module,
            &ambiguous_dependency_symbols,
        )
        .expect("ambiguous dependency attributed chain reserve should still extract");
        assert!(
            ambiguous_dependency_extraction.mode_expansions().is_empty(),
            "ambiguous dependencies remain outside Task64"
        );

        let chain_attributed_rhs_symbols = source_mode_chain_symbol_env(ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("bridge"),
        ));
        let chain_attributed_rhs_module = chain_attributed_rhs_symbols.module_id().clone();
        let chain_attributed_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let chain_attributed_rhs_extraction = extract_builtin_source_reserve_declarations(
            &chain_attributed_rhs,
            chain_attributed_rhs_module.clone(),
            &chain_attributed_rhs_symbols,
        )
        .expect("mode chain ending in attributed RHS reserve should still extract");
        let chain_attributed_b = resolve_visible_type_head(
            &chain_attributed_rhs_symbols,
            &chain_attributed_rhs_module,
            "B",
        )
        .unwrap();
        let chain_attributed_a = resolve_visible_type_head(
            &chain_attributed_rhs_symbols,
            &chain_attributed_rhs_module,
            "A",
        )
        .unwrap();
        assert_eq!(chain_attributed_rhs_extraction.mode_expansions().len(), 2);
        let chain_attributed_b_expansion = chain_attributed_rhs_extraction
            .mode_expansions()
            .get(&chain_attributed_b)
            .expect("terminal attributed RHS expansion should be present");
        assert!(matches!(
            chain_attributed_b_expansion.radix.head,
            TypeHeadInput::BuiltinSet
        ));
        assert_eq!(chain_attributed_b_expansion.attributes.len(), 1);
        assert_eq!(
            chain_attributed_b_expansion.attributes[0].polarity,
            AttributePolarity::Negative
        );
        assert_eq!(
            chain_attributed_rhs_extraction
                .mode_expansions()
                .get(&chain_attributed_a)
                .map(|expansion| &expansion.radix.head),
            Some(&TypeHeadInput::Symbol(chain_attributed_b.clone()))
        );

        let chain_attributed_object_rhs = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedObject),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let chain_attributed_object_rhs_extraction = extract_builtin_source_reserve_declarations(
            &chain_attributed_object_rhs,
            chain_attributed_rhs_module.clone(),
            &chain_attributed_rhs_symbols,
        )
        .expect("mode chain ending in attributed object RHS reserve should still extract");
        let chain_attributed_object_b_expansion = chain_attributed_object_rhs_extraction
            .mode_expansions()
            .get(&chain_attributed_b)
            .expect("terminal attributed object RHS expansion should be present");
        assert!(matches!(
            chain_attributed_object_b_expansion.radix.head,
            TypeHeadInput::BuiltinObject
        ));
        assert_eq!(chain_attributed_object_b_expansion.attributes.len(), 1);
        assert!(
            chain_attributed_object_rhs_extraction
                .mode_expansions()
                .contains_key(&chain_attributed_a)
        );

        let cached_attributed_rhs_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["z"], ReserveTypeShape::QualifiedSymbol("A")),
            ],
        );
        let cached_attributed_rhs_chain_extraction = extract_builtin_source_reserve_declarations(
            &cached_attributed_rhs_chain,
            chain_attributed_rhs_module.clone(),
            &chain_attributed_rhs_symbols,
        )
        .expect("cached attributed RHS chain reserve should still extract");
        assert_eq!(
            cached_attributed_rhs_chain_extraction
                .mode_expansions()
                .len(),
            2
        );
        assert!(
            cached_attributed_rhs_chain_extraction
                .mode_expansions()
                .contains_key(&chain_attributed_b)
        );
        assert!(
            cached_attributed_rhs_chain_extraction
                .mode_expansions()
                .contains_key(&chain_attributed_a)
        );

        let deeper_attributed_rhs_chain_symbols = source_local_symbols_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            &[
                ("empty", SymbolKind::Attribute),
                ("B", SymbolKind::Mode),
                ("A", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
            ],
        );
        let deeper_attributed_rhs_chain_module =
            deeper_attributed_rhs_chain_symbols.module_id().clone();
        let deeper_attributed_rhs_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("A")),
            ],
            vec![
                reserve_item(vec!["x"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("A")),
                reserve_item(vec!["z"], ReserveTypeShape::QualifiedSymbol("C")),
            ],
        );
        let deeper_attributed_rhs_chain_extraction = extract_builtin_source_reserve_declarations(
            &deeper_attributed_rhs_chain,
            deeper_attributed_rhs_chain_module.clone(),
            &deeper_attributed_rhs_chain_symbols,
        )
        .expect("deeper attributed RHS chain reserve should still extract");
        let deeper_attributed_b = resolve_visible_type_head(
            &deeper_attributed_rhs_chain_symbols,
            &deeper_attributed_rhs_chain_module,
            "B",
        )
        .unwrap();
        let deeper_attributed_a = resolve_visible_type_head(
            &deeper_attributed_rhs_chain_symbols,
            &deeper_attributed_rhs_chain_module,
            "A",
        )
        .unwrap();
        let deeper_attributed_c = resolve_visible_type_head(
            &deeper_attributed_rhs_chain_symbols,
            &deeper_attributed_rhs_chain_module,
            "C",
        )
        .unwrap();
        assert!(
            deeper_attributed_rhs_chain_extraction
                .mode_expansions()
                .contains_key(&deeper_attributed_b)
        );
        assert!(
            deeper_attributed_rhs_chain_extraction
                .mode_expansions()
                .contains_key(&deeper_attributed_a)
        );
        assert!(
            !deeper_attributed_rhs_chain_extraction
                .mode_expansions()
                .contains_key(&deeper_attributed_c),
            "deeper attributed RHS chains remain outside the one-edge bridge"
        );

        let attributed_rhs_with_attribute_args = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSetWithAttributeArgs),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let attributed_rhs_with_attribute_args_extraction =
            extract_builtin_source_reserve_declarations(
                &attributed_rhs_with_attribute_args,
                chain_attributed_rhs_module.clone(),
                &chain_attributed_rhs_symbols,
            )
            .expect("attribute-argument attributed RHS chain reserve should still extract");
        assert!(
            attributed_rhs_with_attribute_args_extraction
                .mode_expansions()
                .is_empty(),
            "attribute arguments on the terminal attributed RHS remain outside Task63"
        );

        let attributed_rhs_with_mode_args = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbolWithArgs("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let attributed_rhs_with_mode_args_extraction = extract_builtin_source_reserve_declarations(
            &attributed_rhs_with_mode_args,
            chain_attributed_rhs_module.clone(),
            &chain_attributed_rhs_symbols,
        )
        .expect("argument-bearing attributed RHS chain reserve should still extract");
        assert!(
            attributed_rhs_with_mode_args_extraction
                .mode_expansions()
                .is_empty(),
            "argument-bearing mode dependencies remain outside Task63"
        );

        let contextual_attributed_rhs_chain = mode_chain_reserve_ast(
            source,
            [
                contextual_mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let contextual_attributed_rhs_extraction = extract_builtin_source_reserve_declarations(
            &contextual_attributed_rhs_chain,
            chain_attributed_rhs_module.clone(),
            &chain_attributed_rhs_symbols,
        )
        .expect("contextual attributed RHS chain reserve should still extract");
        assert!(
            contextual_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "definition-local context keeps attributed-RHS chain expansions withheld"
        );

        let parameterized_attributed_rhs_chain = mode_chain_reserve_ast(
            source,
            [
                parameterized_mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let parameterized_attributed_rhs_extraction = extract_builtin_source_reserve_declarations(
            &parameterized_attributed_rhs_chain,
            chain_attributed_rhs_module.clone(),
            &chain_attributed_rhs_symbols,
        )
        .expect("parameterized attributed RHS chain reserve should still extract");
        assert!(
            parameterized_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "parameterized definitions remain outside Task63"
        );

        let recovered_attributed_rhs_chain = mode_chain_reserve_ast(
            source,
            [
                recovered_mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let recovered_attributed_rhs_extraction = extract_builtin_source_reserve_declarations(
            &recovered_attributed_rhs_chain,
            chain_attributed_rhs_module.clone(),
            &chain_attributed_rhs_symbols,
        )
        .expect("recovered attributed RHS chain reserve should still extract");
        assert!(
            recovered_attributed_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "recovered definitions remain outside Task63"
        );

        let imported_attribute_chain_symbols = imported_attribute_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
        );
        let imported_attribute_chain_module = imported_attribute_chain_symbols.module_id().clone();
        let imported_attribute_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let imported_attribute_chain_extraction = extract_builtin_source_reserve_declarations(
            &imported_attribute_chain,
            imported_attribute_chain_module,
            &imported_attribute_chain_symbols,
        )
        .expect("imported-attribute attributed RHS chain reserve should still extract");
        assert!(
            imported_attribute_chain_extraction
                .mode_expansions()
                .is_empty(),
            "imported terminal RHS attributes remain outside Task63"
        );

        let ambiguous_attribute_chain_symbols = ambiguous_attribute_mode_chain_symbol_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
        );
        let ambiguous_attribute_chain_module =
            ambiguous_attribute_chain_symbols.module_id().clone();
        let ambiguous_attribute_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::AttributedSet),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let ambiguous_attribute_chain_extraction = extract_builtin_source_reserve_declarations(
            &ambiguous_attribute_chain,
            ambiguous_attribute_chain_module,
            &ambiguous_attribute_chain_symbols,
        )
        .expect("ambiguous-attribute attributed RHS chain reserve should still extract");
        assert!(
            ambiguous_attribute_chain_extraction
                .mode_expansions()
                .is_empty(),
            "ambiguous terminal RHS attributes remain outside Task63"
        );

        let forward_structure_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let forward_structure_rhs_symbols = source_local_symbols_env(
            forward_structure_rhs_module.clone(),
            &[
                ("Struct", SymbolKind::Structure),
                ("Mode", SymbolKind::Mode),
            ],
        );
        let forward_structure_rhs = mode_chain_reserve_ast_with_trailing_structures(
            source,
            [mode_definition(
                "Mode",
                ReserveTypeShape::QualifiedSymbol("Struct"),
            )],
            ["Struct"],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("Mode"),
            )],
        );
        let forward_structure_rhs_extraction = extract_builtin_source_reserve_declarations(
            &forward_structure_rhs,
            forward_structure_rhs_module,
            &forward_structure_rhs_symbols,
        )
        .expect("forward structure RHS reserve should still extract");
        assert!(
            forward_structure_rhs_extraction
                .mode_expansions()
                .is_empty(),
            "structure RHS expansion payloads require the structure definition to precede the mode definition"
        );

        let forward_dependency = mode_chain_reserve_ast(
            source,
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("A")),
                mode_definition("A", ReserveTypeShape::Builtin("set")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("B"),
            )],
        );
        let forward_extraction =
            extract_builtin_source_reserve_declarations(&forward_dependency, module, &symbols)
                .expect("forward dependency reserve should still extract");
        assert!(
            forward_extraction.mode_expansions().is_empty(),
            "mode expansion chains require each dependency definition to precede its use"
        );

        let three_edge_symbols = source_local_symbols_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            &[
                ("A", SymbolKind::Mode),
                ("B", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
                ("D", SymbolKind::Mode),
            ],
        );
        let three_edge_module = three_edge_symbols.module_id().clone();
        let three_edge_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::Builtin("set")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("A")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("D", ReserveTypeShape::QualifiedSymbol("C")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("D"),
            )],
        );
        let three_edge_extraction = extract_builtin_source_reserve_declarations(
            &three_edge_chain,
            three_edge_module.clone(),
            &three_edge_symbols,
        )
        .expect("three-edge chain reserve should extract");
        let three_edge_a =
            resolve_visible_type_head(&three_edge_symbols, &three_edge_module, "A").unwrap();
        let three_edge_b =
            resolve_visible_type_head(&three_edge_symbols, &three_edge_module, "B").unwrap();
        let three_edge_c =
            resolve_visible_type_head(&three_edge_symbols, &three_edge_module, "C").unwrap();
        let three_edge_d =
            resolve_visible_type_head(&three_edge_symbols, &three_edge_module, "D").unwrap();
        assert_eq!(three_edge_extraction.mode_expansions().len(), 4);
        assert!(
            three_edge_extraction
                .mode_expansions()
                .contains_key(&three_edge_a)
        );
        assert!(
            three_edge_extraction
                .mode_expansions()
                .contains_key(&three_edge_b)
        );
        assert!(
            three_edge_extraction
                .mode_expansions()
                .contains_key(&three_edge_c)
        );
        assert!(
            three_edge_extraction
                .mode_expansions()
                .contains_key(&three_edge_d),
            "task 73 admits three local-mode dependency edges beyond the reserve head"
        );

        let four_edge_symbols = source_local_symbols_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            &[
                ("A", SymbolKind::Mode),
                ("B", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
                ("D", SymbolKind::Mode),
                ("E", SymbolKind::Mode),
            ],
        );
        let four_edge_module = four_edge_symbols.module_id().clone();
        let four_edge_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::Builtin("set")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("A")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("D", ReserveTypeShape::QualifiedSymbol("C")),
                mode_definition("E", ReserveTypeShape::QualifiedSymbol("D")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("E"),
            )],
        );
        let four_edge_extraction = extract_builtin_source_reserve_declarations(
            &four_edge_chain,
            four_edge_module.clone(),
            &four_edge_symbols,
        )
        .expect("four-edge chain reserve should extract");
        let four_edge_a =
            resolve_visible_type_head(&four_edge_symbols, &four_edge_module, "A").unwrap();
        let four_edge_b =
            resolve_visible_type_head(&four_edge_symbols, &four_edge_module, "B").unwrap();
        let four_edge_c =
            resolve_visible_type_head(&four_edge_symbols, &four_edge_module, "C").unwrap();
        let four_edge_d =
            resolve_visible_type_head(&four_edge_symbols, &four_edge_module, "D").unwrap();
        let four_edge_e =
            resolve_visible_type_head(&four_edge_symbols, &four_edge_module, "E").unwrap();
        assert_eq!(four_edge_extraction.mode_expansions().len(), 5);
        for symbol in [
            &four_edge_a,
            &four_edge_b,
            &four_edge_c,
            &four_edge_d,
            &four_edge_e,
        ] {
            assert!(
                four_edge_extraction.mode_expansions().contains_key(symbol),
                "task 74 admits structurally valid bare builtin-terminal local-mode chains"
            );
        }

        let long_chain_symbols = source_local_symbols_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            &[
                ("A", SymbolKind::Mode),
                ("B", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
                ("D", SymbolKind::Mode),
                ("E", SymbolKind::Mode),
                ("F", SymbolKind::Mode),
                ("G", SymbolKind::Mode),
            ],
        );
        let long_chain_module = long_chain_symbols.module_id().clone();
        let long_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::Builtin("set")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("A")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("D", ReserveTypeShape::QualifiedSymbol("C")),
                mode_definition("E", ReserveTypeShape::QualifiedSymbol("D")),
                mode_definition("F", ReserveTypeShape::QualifiedSymbol("E")),
                mode_definition("G", ReserveTypeShape::QualifiedSymbol("F")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("G"),
            )],
        );
        let long_chain_extraction = extract_builtin_source_reserve_declarations(
            &long_chain,
            long_chain_module.clone(),
            &long_chain_symbols,
        )
        .expect("long chain reserve should extract");
        let long_chain_g =
            resolve_visible_type_head(&long_chain_symbols, &long_chain_module, "G").unwrap();
        assert_eq!(long_chain_extraction.mode_expansions().len(), 7);
        assert!(
            long_chain_extraction
                .mode_expansions()
                .contains_key(&long_chain_g),
            "task 74 is structural and not capped at four local-mode dependency edges"
        );

        let cached_deeper_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let cached_deeper_symbols = source_local_symbols_env(
            cached_deeper_module.clone(),
            &[
                ("A", SymbolKind::Mode),
                ("B", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
                ("D", SymbolKind::Mode),
                ("E", SymbolKind::Mode),
            ],
        );
        let cached_deeper_chain = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::Builtin("set")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("A")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("D", ReserveTypeShape::QualifiedSymbol("C")),
                mode_definition("E", ReserveTypeShape::QualifiedSymbol("D")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("D")),
                reserve_item(vec!["z"], ReserveTypeShape::QualifiedSymbol("E")),
            ],
        );
        let cached_deeper_extraction = extract_builtin_source_reserve_declarations(
            &cached_deeper_chain,
            cached_deeper_module.clone(),
            &cached_deeper_symbols,
        )
        .expect("cached deeper chain reserve should still extract");
        let cached_a =
            resolve_visible_type_head(&cached_deeper_symbols, &cached_deeper_module, "A").unwrap();
        let cached_b =
            resolve_visible_type_head(&cached_deeper_symbols, &cached_deeper_module, "B").unwrap();
        let cached_c =
            resolve_visible_type_head(&cached_deeper_symbols, &cached_deeper_module, "C").unwrap();
        let cached_d =
            resolve_visible_type_head(&cached_deeper_symbols, &cached_deeper_module, "D").unwrap();
        let cached_e =
            resolve_visible_type_head(&cached_deeper_symbols, &cached_deeper_module, "E").unwrap();
        assert!(
            cached_deeper_extraction
                .mode_expansions()
                .contains_key(&cached_a)
        );
        assert!(
            cached_deeper_extraction
                .mode_expansions()
                .contains_key(&cached_b)
        );
        assert!(
            cached_deeper_extraction
                .mode_expansions()
                .contains_key(&cached_c)
        );
        assert!(
            cached_deeper_extraction
                .mode_expansions()
                .contains_key(&cached_d),
            "cached three-edge expansion payloads remain available for the supported reserve"
        );
        assert!(
            cached_deeper_extraction
                .mode_expansions()
                .contains_key(&cached_e),
            "cached three-edge expansion payloads may feed a structurally valid four-edge chain"
        );

        let cached_structure_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let cached_structure_rhs_symbols = source_local_symbols_env(
            cached_structure_rhs_module.clone(),
            &[
                ("Struct", SymbolKind::Structure),
                ("B", SymbolKind::Mode),
                ("A", SymbolKind::Mode),
            ],
        );
        let cached_structure_rhs_chain = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["z"], ReserveTypeShape::QualifiedSymbol("A")),
            ],
        );
        let cached_structure_rhs_extraction = extract_builtin_source_reserve_declarations(
            &cached_structure_rhs_chain,
            cached_structure_rhs_module.clone(),
            &cached_structure_rhs_symbols,
        )
        .expect("cached structure RHS chain reserve should still extract");
        let cached_structure_b = resolve_visible_type_head(
            &cached_structure_rhs_symbols,
            &cached_structure_rhs_module,
            "B",
        )
        .unwrap();
        let cached_structure_a = resolve_visible_type_head(
            &cached_structure_rhs_symbols,
            &cached_structure_rhs_module,
            "A",
        )
        .unwrap();
        assert!(
            cached_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&cached_structure_b)
        );
        assert!(
            cached_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&cached_structure_a),
            "cached direct structure-RHS expansion should feed the one-edge structure-RHS chain"
        );

        let cached_forward_structure_rhs_chain = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
            ],
            vec![
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["z"], ReserveTypeShape::QualifiedSymbol("A")),
            ],
        );
        let cached_forward_structure_rhs_extraction = extract_builtin_source_reserve_declarations(
            &cached_forward_structure_rhs_chain,
            cached_structure_rhs_module.clone(),
            &cached_structure_rhs_symbols,
        )
        .expect("cached forward structure-RHS chain reserve should still extract");
        assert!(
            cached_forward_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&cached_structure_b)
        );
        assert!(
            !cached_forward_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&cached_structure_a),
            "cached direct structure-RHS payloads must still prove the dependency definition precedes the dependent mode"
        );

        let cached_deeper_structure_rhs_module =
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let cached_deeper_structure_rhs_symbols = source_local_symbols_env(
            cached_deeper_structure_rhs_module.clone(),
            &[
                ("Struct", SymbolKind::Structure),
                ("B", SymbolKind::Mode),
                ("A", SymbolKind::Mode),
                ("C", SymbolKind::Mode),
            ],
        );
        let cached_deeper_structure_rhs_chain = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
                mode_definition("C", ReserveTypeShape::QualifiedSymbol("A")),
            ],
            vec![
                reserve_item(vec!["w"], ReserveTypeShape::QualifiedSymbol("B")),
                reserve_item(vec!["y"], ReserveTypeShape::QualifiedSymbol("A")),
                reserve_item(vec!["z"], ReserveTypeShape::QualifiedSymbol("C")),
            ],
        );
        let cached_deeper_structure_rhs_extraction = extract_builtin_source_reserve_declarations(
            &cached_deeper_structure_rhs_chain,
            cached_deeper_structure_rhs_module.clone(),
            &cached_deeper_structure_rhs_symbols,
        )
        .expect("cached deeper structure-RHS chain reserve should still extract");
        let cached_deeper_structure_b = resolve_visible_type_head(
            &cached_deeper_structure_rhs_symbols,
            &cached_deeper_structure_rhs_module,
            "B",
        )
        .unwrap();
        let cached_deeper_structure_a = resolve_visible_type_head(
            &cached_deeper_structure_rhs_symbols,
            &cached_deeper_structure_rhs_module,
            "A",
        )
        .unwrap();
        let cached_deeper_structure_c = resolve_visible_type_head(
            &cached_deeper_structure_rhs_symbols,
            &cached_deeper_structure_rhs_module,
            "C",
        )
        .unwrap();
        assert!(
            cached_deeper_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&cached_deeper_structure_b)
        );
        assert!(
            cached_deeper_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&cached_deeper_structure_a)
        );
        assert!(
            !cached_deeper_structure_rhs_extraction
                .mode_expansions()
                .contains_key(&cached_deeper_structure_c),
            "cached one-edge structure-RHS payloads must not let a deeper chain bypass the cap"
        );

        let duplicate_mode_definition = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let duplicate_mode_extraction = extract_builtin_source_reserve_declarations(
            &duplicate_mode_definition,
            cached_structure_rhs_module.clone(),
            &cached_structure_rhs_symbols,
        )
        .expect("duplicate mode definition reserve should still extract");
        assert!(
            duplicate_mode_extraction.mode_expansions().is_empty(),
            "structure-RHS chains require a unique preceding mode definition for every mode head"
        );

        let duplicate_structure_definition = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct", "Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let duplicate_structure_extraction = extract_builtin_source_reserve_declarations(
            &duplicate_structure_definition,
            cached_structure_rhs_module.clone(),
            &cached_structure_rhs_symbols,
        )
        .expect("duplicate structure definition reserve should still extract");
        assert!(
            duplicate_structure_extraction.mode_expansions().is_empty(),
            "structure-RHS chains require a unique preceding structure definition for the terminal structure head"
        );

        let contextual_structure_rhs_chain = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                contextual_mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let contextual_structure_extraction = extract_builtin_source_reserve_declarations(
            &contextual_structure_rhs_chain,
            cached_structure_rhs_module.clone(),
            &cached_structure_rhs_symbols,
        )
        .expect("contextual structure-RHS chain reserve should still extract");
        assert!(
            contextual_structure_extraction.mode_expansions().is_empty(),
            "definition-local context keeps structure-RHS chain expansions withheld"
        );

        let parameterized_structure_rhs_chain = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                parameterized_mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let parameterized_structure_extraction = extract_builtin_source_reserve_declarations(
            &parameterized_structure_rhs_chain,
            cached_structure_rhs_module.clone(),
            &cached_structure_rhs_symbols,
        )
        .expect("parameterized structure-RHS chain reserve should still extract");
        assert!(
            parameterized_structure_extraction
                .mode_expansions()
                .is_empty(),
            "parameterized mode definitions remain outside the source-derived structure-RHS chain slice"
        );

        let recovered_structure_rhs_chain = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                recovered_mode_definition("B", ReserveTypeShape::QualifiedSymbol("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let recovered_structure_extraction = extract_builtin_source_reserve_declarations(
            &recovered_structure_rhs_chain,
            cached_structure_rhs_module.clone(),
            &cached_structure_rhs_symbols,
        )
        .expect("recovered structure-RHS chain reserve should still extract");
        assert!(
            recovered_structure_extraction.mode_expansions().is_empty(),
            "recovered mode definitions remain outside the source-derived structure-RHS chain slice"
        );

        let argument_structure_rhs_chain = mode_chain_reserve_ast_with_structures(
            source,
            ["Struct"],
            [
                mode_definition("B", ReserveTypeShape::QualifiedSymbolWithArgs("Struct")),
                mode_definition("A", ReserveTypeShape::QualifiedSymbol("B")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let argument_structure_extraction = extract_builtin_source_reserve_declarations(
            &argument_structure_rhs_chain,
            cached_structure_rhs_module.clone(),
            &cached_structure_rhs_symbols,
        )
        .expect("argument-bearing structure-RHS chain reserve should still extract");
        assert!(
            argument_structure_extraction.mode_expansions().is_empty(),
            "argument-bearing RHS symbols remain outside the source-derived structure-RHS chain slice"
        );

        let cyclic_symbols = source_local_symbols_env(
            ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge")),
            &[("A", SymbolKind::Mode)],
        );
        let cyclic_module = cyclic_symbols.module_id().clone();
        let cyclic_chain = mode_chain_reserve_ast(
            source,
            [mode_definition("A", ReserveTypeShape::QualifiedSymbol("A"))],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("A"),
            )],
        );
        let cyclic_extraction = extract_builtin_source_reserve_declarations(
            &cyclic_chain,
            cyclic_module,
            &cyclic_symbols,
        )
        .expect("cyclic chain reserve should still extract");
        assert!(
            cyclic_extraction.mode_expansions().is_empty(),
            "cyclic local-mode RHS dependencies must not produce partial chain expansions"
        );

        let ambiguous_symbols = ambiguous_mode_chain_symbol_env(ResolverModuleId::new(
            PackageId::new("test"),
            ModulePath::new("bridge"),
        ));
        let ambiguous_module = ambiguous_symbols.module_id().clone();
        let ambiguous_dependency = mode_chain_reserve_ast(
            source,
            [
                mode_definition("A", ReserveTypeShape::Builtin("set")),
                mode_definition("B", ReserveTypeShape::QualifiedSymbol("A")),
            ],
            vec![reserve_item(
                vec!["z"],
                ReserveTypeShape::QualifiedSymbol("B"),
            )],
        );
        let ambiguous_extraction = extract_builtin_source_reserve_declarations(
            &ambiguous_dependency,
            ambiguous_module,
            &ambiguous_symbols,
        )
        .expect("ambiguous dependency reserve should still extract");
        assert!(
            ambiguous_extraction.mode_expansions().is_empty(),
            "ambiguous local-mode RHS dependencies must not produce partial chain expansions"
        );
    }

    #[test]
    fn source_reserve_bridge_assembles_declaration_checked_resolved_typed_ast_handoff() {
        let source_id = source_id(94);
        let ast = reserve_ast(
            source_id,
            vec![
                reserve_item(vec!["x", "y"], ReserveTypeShape::Builtin("set")),
                reserve_item(vec!["z"], ReserveTypeShape::Builtin("object")),
            ],
        );
        let module = ResolverModuleId::new(PackageId::new("test"), ModulePath::new("bridge"));
        let symbols = SymbolEnv::new(module.clone(), SymbolEnvIndexes::default());
        let source_reserve =
            extract_builtin_source_reserve_declarations(&ast, module.clone(), &symbols)
                .expect("builtin reserve declarations should extract");

        let handoff = assemble_source_checker_handoff(&symbols, &source_reserve)
            .expect("source-derived checker handoff should reach ResolvedTypedAst");
        let resolved = &handoff.resolved;

        assert_eq!(handoff.binding_env.bindings().len(), 3);
        let module_context = handoff
            .binding_env
            .contexts()
            .get(source_reserve.module_context())
            .expect("module binding context should exist");
        assert_eq!(
            module_context.bindings,
            vec![BindingId::new(0), BindingId::new(1), BindingId::new(2)]
        );
        assert_eq!(module_context.visible_bindings, module_context.bindings);
        assert_eq!(handoff.declarations.declarations().len(), 3);
        assert_eq!(handoff.declarations.contexts().len(), 1);
        assert!(handoff.declarations.diagnostics().is_empty());
        assert_eq!(handoff.typed_ast.contexts().len(), 1);
        assert_eq!(resolved.nodes().len(), 7);
        assert_eq!(resolved.expr_metadata().len(), 3);
        assert!(resolved.diagnostics().is_empty());
        let summary = ResolvedTypedAstSummary::from_ast(resolved);
        assert_eq!(summary.source_id(), source_id);
        assert_eq!(summary.module_id(), resolved.module_id());
        assert!(
            summary.checker_sites().is_empty(),
            "successful reserve-only source payload should be summary-readable without checker recovery sites"
        );
        assert_ne!(source_reserve.type_node(0), source_reserve.type_node(1));
        assert_eq!(
            source_reserve.bindings()[0].type_range,
            source_reserve.bindings()[1].type_range
        );
        for index in 0..source_reserve.bindings().len() {
            let type_node = resolved
                .nodes()
                .node(ResolvedTypedNodeId::new(
                    source_reserve.type_node(index).index(),
                ))
                .expect("resolved type node should be present");
            match &type_node.kind {
                ResolvedTypedNodeKind::SourcePreserved { role } => {
                    assert_eq!(role.as_str(), "source.reserve.type_expression");
                }
                other => panic!("unexpected resolved type node kind: {other:?}"),
            }
            assert!(type_node.final_type.is_some());
            let declaration_node = resolved
                .nodes()
                .node(ResolvedTypedNodeId::new(
                    source_reserve.declaration_node(index).index(),
                ))
                .expect("resolved declaration node should be present");
            match &declaration_node.kind {
                ResolvedTypedNodeKind::SourcePreserved { role } => {
                    assert_eq!(role.as_str(), "source.reserve.declaration");
                }
                other => panic!("unexpected resolved declaration node kind: {other:?}"),
            }
            assert_eq!(declaration_node.children.len(), 1);
            assert!(declaration_node.final_type.is_some());
            let expr = mizar_checker::resolved_typed_ast::ExprId::new(format!(
                "source.reserve.declaration.{index}"
            ));
            let metadata = resolved
                .expr_metadata()
                .get_by_expr(&expr)
                .expect("expression metadata should be present");
            assert!(metadata.final_type.is_some());
            assert_eq!(metadata.local_context, Some(LocalTypeContextId::new(0)));
        }
        assert!(resolved.debug_text().contains("source.reserve.declaration"));
        assert!(
            resolved
                .debug_text()
                .contains("source.reserve.type_expression")
        );
    }
