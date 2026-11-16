# Old whole-box graph

Nodes: 3013; edges: 3548489; speed bound: 1.533322324673778; jump apex: 1.252203340253724.

Julien regression: [-84, -29, -102] -> [-85, -32, -96] -> [-84, -30, -92]. Both edges retained.

- Skipping initial ladder: None
- Skipping north ice: Some([[-107, -60, -107], [-106, -59, -107], [-106, -57, -107], [-106, -55, -107], [-106, -53, -107], [-106, -51, -107], [-106, -49, -107], [-106, -47, -107], [-106, -45, -107], [-106, -43, -107], [-106, -41, -107], [-74, -48, -99], [-84, -30, -92], [-84, -28, -92], [-84, -26, -92], [-84, -25, -88]])
- Skipping main slime: Some([[-107, -60, -107], [-106, -58, -107], [-106, -56, -107], [-106, -54, -107], [-106, -52, -107], [-106, -50, -107], [-106, -48, -107], [-106, -46, -107], [-106, -44, -107], [-106, -42, -107], [-106, -40, -107], [-106, -38, -107], [-106, -36, -107], [-106, -34, -107], [-106, -32, -107], [-106, -30, -107], [-102, -29, -101], [-84, -28, -92], [-84, -26, -92], [-84, -25, -88]])
- Skipping final ladder: Some([[-107, -60, -107], [-106, -59, -107], [-106, -57, -107], [-106, -55, -107], [-106, -53, -107], [-106, -51, -107], [-106, -49, -107], [-106, -47, -107], [-106, -45, -107], [-106, -43, -107], [-106, -41, -107], [-74, -48, -99], [-79, -30, -92], [-79, -28, -92], [-79, -26, -92], [-84, -25, -88]])

## Every skipping path (finite complete description)

The graph contains cycles, so its A-to-B walks cannot be listed one per line. `old-graph.tsv` lists every node and edge. Each line below lists the complete A/B core after removing an element; every A-to-B walk in that induced subgraph is a hole, and the union of these four languages is exactly every walk skipping an intended element. Successor runs and node runs are inclusive. B is terminal. These are conservative graph paths, not claims that each is physically executable.

- Omit initial ladder: 0 core nodes, 0 core edges; core IDs are in `old-holes.tsv`, mask 1.
- Omit north ice: 2664 core nodes, 2807455 core edges; core IDs are in `old-holes.tsv`, mask 2.
- Omit main slime: 2690 core nodes, 2878192 core edges; core IDs are in `old-holes.tsv`, mask 4.
- Omit final ladder: 2680 core nodes, 2851988 core edges; core IDs are in `old-holes.tsv`, mask 8.

Local excluded edges: 10043; minimum exclusion margins (height, horizontal distance, component deficit): [0.5996604954373232, 0.00013208007280951506, 1.0].
