/** @typedef {{revision:string, dirtyDiffSha256:string|null}} Source */
/** @typedef {{id:string, runner:string, level:string, platform:string, pointerWidth?:number, binarySha256:string, testBinarySha256?:string}} DiscoveredTest */
/** @typedef {{testId:string, attempt:number, status:string, durationMs:number, timestamp:string, binarySha256:string, testBinarySha256?:string, platform:string, toolchain:string, device:string, environment:{valid:boolean,reasons:string[]}, artifacts:string[], output?:string}} Result */
/** @typedef {{platform:string, level:string, pointerWidth?:number}} Requirement */
/** @typedef {{id:string, expected:string, invariants:string[], ownerTask:string, requirements:Requirement[], testIds:string[], automation:string, gaps:{reason:string,ownerTask:string}[]}} Case */
/** @typedef {{schemaVersion:number,id:string,description:string,sourceRevision:string,cases:Case[]}} Feature */
/** @typedef {{schemaVersion:number,source:Source,tests:DiscoveredTest[]}} Discovery */
/** @typedef {{schemaVersion:number,source:Source,results:Result[]}} Execution */
export {};
