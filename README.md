# Mizar Evolution

***Building the Next Generation of Formal Mathematics Environment Using 50 Years of Heritage***

Mizar, which began development in 1973, is one of the oldest interactive theorem provers (ITPs). For over five decades, it has played a vital role in formalization of mathematics. Throughout its long history, Mizar has built an extensive formal library called the Mizar Mathematical Library (MML). This library has significantly contributed to the precise description and verification of mathematical knowledge.

However, Mizar's long history has also led to certain challenges. The system lacks modern programming language features such as generics and advanced type systems. It also does not fully utilize recent advanced technologies appeared in automated theorem provers. These outdated language designs create significant obstacles for library developers, making it difficult to formalize new mathematical concepts and expand existing libraries efficiently. Moreover, the user interface and development environment have fallen behind current standards, creating barriers for new users.

The main goal of this redesign is to address these issues while maximizing the use of Mizar's rich mathematical resources. We plan to introduce modern programming language concepts to enable more expressive and efficient formalization. By adding features like template, namespace, and more flexible module systems, we aim to give developers greater freedom in their work.

Simultaneously, we will integrate the latest automated proof technologies. This will enhance the efficiency and automation of theorem-proving processes. As a result, library developers can work more productively and formalize more complex mathematical concepts. We also aim to provide a modern development environment and tools, significantly improving the user experience and encouraging new users to join.

The redesign includes numerous improvements: updating the language design, integrating automated proof technologies, enhancing the module system, introduction of tactics and algorithm validation functions, modernizing the development environment. With these changes, Mizar will evolve into a next-generation environment for formal mathematics.

These improvements are expected to accelerate library development, attract a wider user base, promote the formalization of modern mathematics, strengthen integration with software verification, and expand its use in education and research.

In implementing these changes, we plan to introduce new features gradually, considering compatibility with the existing MML. We will promote open-source development to leverage community expertise and speed up progress. We will also design the system to handle large-scale MML and prepare for future expansions.

This redesign will help Mizar evolve into a next-generation formal mathematics environment. It will build on its 50-year heritage while incorporating modern technology and design principles. We expect this to open a new era in formal mathematics and significantly contribute to the advancement of mathematical research and formal methods.

This is the place to discuss future directions for the Mizar language / system / library specifications. We welcome your constructive opinions. Please feel free to participate in the discussion.

## Project Layout

```
mizar-evo/
├── README.md
├── LICENSE
├── logo/
│   └── evo0.png
│
├── doc/
│   ├── spec/              ← Language specification (external)
│   │   ├── README.md
│   │   ├── en/            Canonical English language specs
│   │   │   ├── 00.index.md
│   │   │   ├── 01.introduction.md
│   │   │   ├── ...
│   │   │   └── 24.documentation_generation.md
│   │   └── ja/            Japanese companion language specs
│   │
│   ├── design/            ← Implementation specification (internal)
│   │   ├── README.md
│   │   ├── architecture/          Cross-cutting design decisions
│   │   │   ├── README.md
│   │   │   ├── reasoning_boundary.md
│   │   │   ├── atp_interface_protocol.md
│   │   │   └── atp_backend_integration.md
│   │   └── <crate>/               Maps 1:1 to Rust source files
│   │
│   └── idea/              ← Design notes and ideas
│       ├── algorithm_verification.md
│       └── property_verification_and_atp.md
│
└── crates/                ← Rust workspace (to be created)
```

### AI-Driven Development

Internal specifications in `doc/design/` map 1:1 to Rust source files:

```
doc/design/<crate>/<module>.md  →  crates/<crate>/src/<module>.rs
```

This enables spec-driven implementation where each module can be developed and verified against its specification independently.

Current crate design roots and crate-wide autonomous development evidence are
indexed in [`doc/design/README.md`](./doc/design/README.md). The completed
lexer crate evidence is available from
[`doc/design/mizar-lexer/`](./doc/design/mizar-lexer/README.md), including its
[autonomous crate plan](./doc/design/mizar-lexer/en/00.crate_plan.md) and
[crate exit report](./doc/design/mizar-lexer/en/crate_exit_report.md).
