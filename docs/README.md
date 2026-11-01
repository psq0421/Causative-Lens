# CausativeLens documentation

> **Documentation is a second executable: it tells the next engineer where the evidence lives.**

Choose a language: [English (US)](README.md) · [简体中文](README_CN.md) · [繁體中文](README_TW.md).

## Guides

- [v0.2.5 first observation](guide/monitoring-start.md) · [简体中文](guide/monitoring-start_CN.md) · [繁體中文](guide/monitoring-start_TW.md)
- [Quick start](quick-start.md) · [简体中文](快速入门.md) · [繁體中文](快速入門.md)
- [17-product integration tutorial](guide/runtime-integration.md) · [简体中文](guide/runtime-integration_CN.md)
- [Contributing](guide/contributing.md) · [简体中文](guide/contributing_CN.md) · [繁體中文](guide/contributing_TW.md)
- [Language support](guide/language-support.md) · [简体中文](guide/language-support_CN.md) · [繁體中文](guide/language-support_TW.md)

## Architecture

- [Architecture overview](architecture/README.md) · [简体中文](architecture/README_CN.md) · [繁體中文](architecture/README_TW.md)
- [Architecture audit](plan/architecture-audit-3.md)
- [Runtime and concurrency audit](plan/architecture-stack-audit.md)
- [Data model audit](plan/data-model-audit-6.md)

## Contracts and specifications

- [ETW collection](spec/etw-7.md) · [Harness adapters](spec/harness-8.md)
- [Alignment](spec/alignment-9.md) · [Query and replay](spec/query-11.md)
- [IPC and security](spec/ipc-security-12.md) · [Runtime adapters](spec/runtime-adapters.md)
- [RPC observe contract](spec/rpc-observe.md) · [Dependency policy](spec/dependency-policy.md)
- [Observe monitoring contract](spec/monitoring-0.1.1.en-US.md) · [简体中文](spec/monitoring-0.1.1.md) · [繁體中文](spec/monitoring-0.1.1_TW.md)
- [Bilingual contract index](reference/contracts.md)

## Plans and task cards

- [Implementation plan](plan/implementation-plan.md)
- [Plan index and gate table](plan/README.en-US.md) · [简体中文](plan/README.md) · [繁體中文](plan/README_TW.md)
- [GUI UX specification](plan/gui-ux-spec.md)
- [Installer specification](plan/installer-spec.md)
- [Installer build notes](../installer/README.md) · [简体中文](../installer/README_CN.md) · [繁體中文](../installer/README_TW.md)
- [Test plan](plan/test-plan.md)
- [Open questions](plan/open-questions.md)
- [Task cards](plan/task-cards/README.en-US.md) · [简体中文](plan/task-cards/README.md) · [繁體中文](plan/task-cards/README_TW.md)
- [Gate reports](plan/gate-reports/)

## Release and appendices

- [Release gate G7](release/g7-release-gate.md) · [简体中文](release/g7-release-gate_CN.md) · [繁體中文](release/g7-release-gate_TW.md)
- [v0.2.5 release notes](release/v0.2.5.en-US.md) · [简体中文](release/v0.2.5.md) · [繁體中文](release/v0.2.5_TW.md)
- [v0.2.5 validation](release/v0.2.5-validation.en-US.md) · [简体中文](release/v0.2.5-validation.md) · [繁體中文](release/v0.2.5-validation_TW.md)
- [Changelog](release/CHANGELOG.md) · [简体中文](release/CHANGELOG_CN.md) · [繁體中文](release/CHANGELOG_TW.md)
- [v0.2.0 evidence-loop plan](plan/v0.2.0.en-US.md) · [简体中文](plan/v0.2.0.md)
- [Capture fast-path refinement](plan/capture-fast-path.en-US.md) · [简体中文](plan/capture-fast-path.md)
- [Field reference](reference/field-reference.md) · [简体中文](reference/field-reference_CN.md) · [繁體中文](reference/field-reference_TW.md)
- [Compatibility matrix](reference/compatibility.md) · [简体中文](reference/compatibility_CN.md) · [繁體中文](reference/compatibility_TW.md)
- [Appendix map](reference/appendices.md) · [简体中文](reference/appendices_CN.md) · [繁體中文](reference/appendices_TW.md)

The detailed plan and specification files retain their contract identifiers (`CL-BLUEPRINT-001`, `.cllens`, `.clspool`). Each guide links to its language companions so reviewers can find the authoritative source without guessing.
