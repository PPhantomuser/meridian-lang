# Security Policy

Security is a primary concern for the Meridian language, particularly around the `unsafe` FFI boundaries, the ARC memory cycle collector, and the Cranelift JIT/native compiler backend.

## Supported Versions

Currently, Meridian is in pre-1.0 development. We provide security patches for the `main` branch.

| Version | Supported          |
| ------- | ------------------ |
| `main`  | :white_check_mark: |
| `< 1.0` | :x:                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them using **GitHub's Private Vulnerability Reporting** feature. If that feature is unavailable, please use a secure GitHub contact method to reach the Lead Maintainer privately. 

When reporting a vulnerability, please provide:
- A description of the vulnerability and its potential impact.
- Steps to reproduce the issue (a minimal reproducible example `.mer` script is highly appreciated).
- Any relevant system information (OS, Rust version, Meridian commit hash).

We will endeavor to respond to your report promptly, triage the issue, and work with you to develop a patch and coordinate a disclosure timeline.

## Specific Areas of Interest
We are particularly interested in reports concerning:
- Memory safety violations (use-after-free, double-free) in the ARC implementation or cycle collector.
- Soundness holes in the `SemanticAnalyzer` (type system escapes).
- Escapes or vulnerabilities in the `meridian_backend_cranelift` codegen.
- Bugs that allow safe Meridian code to trigger Undefined Behavior (UB) without an explicit `unsafe` block.
