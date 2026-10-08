# Collection receiver validation owns its actual GC reference

A native Map/Set method validates the exact MapObject/SetObject reference before
argument conversion or property observation. A native iterator validates its
exact concrete iterator reference before reading the retained collection edge.
A Proxy is an independent reference and never forwards this internal-slot gate.

Only successful validation produces the typed record accepted by the remaining
algorithm. Wrong receivers produce a whole TypeError in the executing function's
Realm. This gate cannot request weak runtime capabilities, unwrap a Proxy, read
a manual brand word, or select an integer heap layout. SameValueZero key lookup
and closed output kinds consume their own typed owners after this gate.

The prior raw NonObject/MissingInternalSlots spelling/count mirrors are retired
with their exact preimages preserved. Meaningful Engine controls retain direct
brand rejection, absence of Proxy Gets and borrowed-Realm errors; existing CLI
receiver fixtures remain intact. New native controls are unrun, the atomic
cutover remains pending and no conformance count or task acceptance is changed.
