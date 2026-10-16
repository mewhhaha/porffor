This checkpoint records a private candidate. Its compiler fingerprint and selected-source commitment identify the tested source; they are separate from the identity of this documentation-only Git commit.

All six targeted real Test262 executions now report Success under the unchanged 60,000 ms case deadline. The two JSON modes took 53.436 s and 52.934 s. ModuleSource prototype-chain and Reflect arguments-list cases also pass in both modes. Their historical baseline failures remain in the proof.

The diagnostic with 96 eval calls uses the same Conformance profile as its predecessor. Execution and completion took 1.386 s, compared with 5.686 s before: a 4.10× improvement in that measured interval. The run recorded 94 native-template hits and two misses. Cold module compilation took a separate 31.424 s. The unchanged Engine did not measure internal callback phases. This result does not establish that the 65,536-iteration probes finish within 60 s.

The complete IR unit suite remains red: 1,577 passed and five failed. The complete IR integration target also remains red: 732 passed and two failed. Neither run ignored tests. Their actual inventories, raw stdout and all seven failing IDs are embedded in the proof. Seven assertion repairs are authored but unrun; they do not change the captured outcomes.

Five other unit harnesses and 13 other integration harnesses were unrun at capture. The separate six-target AOT integration check also retains its missing-fixture failure. A namespace repair does not establish semantic test acceptance.

Run the adjacent offline reader with Python 3 and the JSON proof. It needs no private files, compiler, native library or network. It checks raw-byte digests, source and protected-input fence equality, measured intervals, all six real outcomes and all seven IR failures.

Full-suite, ordinary Cargo, combined published-source identity and task gates remain open. Generated status numbers and all historical README and CONTINUE content are preserved.
