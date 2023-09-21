import * as wasm from "./asteroids_bg.wasm";
import { __wbg_set_wasm } from "./asteroids_bg.js";
__wbg_set_wasm(wasm);
export * from "./asteroids_bg.js";
