import { appendLogLine } from './log.js';

declare global {
    // Optional test/dev override for loading binaries from a local checkout.
    // Keep this global so the deployed HTML can stay config-free.
    var ROBIN_WASM_BINARIES_BASE: string | undefined;
    var rh_rpc_resolve: ((
        id: number,
        status: number,
        contentType: string,
        body: string | Uint8Array,
    ) => void) | undefined;
    var robinRpc: ((method: string, params?: unknown) => Promise<unknown>) | undefined;
}

type BuildSelection = {
    readonly short: string;
    readonly source: 'latest' | 'replay';
};

type BuildManifest = {
    readonly commit?: unknown;
    readonly short?: unknown;
};

type RobinWasmModule = {
    readonly default: (moduleOrPath?: string | URL | Request) => Promise<unknown>;
    readonly wasm_boot: (datadir: Uint8Array) => void;
    readonly wasm_preload_asset?: (path: string, bytes: Uint8Array) => void;
    readonly rh_rpc_enqueue?: (id: number, json: Uint8Array) => void;
};

type PreloadEntry = string | {
    readonly path?: unknown;
    readonly url?: unknown;
};

const DEFAULT_BINARIES_BASE = import.meta.env.DEV
    ? window.location.origin
    : 'https://phiresky.github.io/robin-hood-the-legend-of-sherwood-remake-binaries';
const BINARIES_BASE =
    globalThis.ROBIN_WASM_BINARIES_BASE ??
    DEFAULT_BINARIES_BASE;
const WASM_BUILDS_BASE = `${BINARIES_BASE}/wasm`;
const HASH_RE = /^[0-9a-f]{7,40}$/i;
const COMPACT_REPLAY_RE = /^rhrec-([0-9a-f]{7,40})-/i;

const logEl = document.querySelector<HTMLDivElement>('#log');
if (logEl === null) {
    throw new Error('main.ts: missing #log element in index.html');
}

const logOk = (t: string): void => appendLogLine(logEl, t);
const logErr = (t: string): void => appendLogLine(logEl, t, 'err');
const rpcPending = new Map<number, {
    readonly resolve: (value: unknown) => void;
    readonly reject: (reason?: unknown) => void;
}>();
let nextRpcId = 1;

globalThis.rh_rpc_resolve = (
    id: number,
    status: number,
    contentType: string,
    body: string | Uint8Array,
): void => {
    const pending = rpcPending.get(id);
    if (pending === undefined) {
        logErr(`[rpc ${id}] reply for unknown request`);
        return;
    }
    rpcPending.delete(id);
    const value = parseRpcBody(contentType, body);
    if (status >= 200 && status < 300) {
        pending.resolve(value);
    } else {
        const msg = typeof value === 'object' && value !== null && 'error' in value
            ? String((value as { error: unknown }).error)
            : `RPC ${id} failed with status ${status}`;
        pending.reject(new Error(msg));
    }
};

function parseRpcBody(contentType: string, body: string | Uint8Array): unknown {
    if (body instanceof Uint8Array) {
        return body;
    }
    if (contentType.includes('json')) {
        return JSON.parse(body);
    }
    return body;
}

async function fetchJson(url: string): Promise<BuildManifest> {
    const resp = await fetch(url, { cache: 'no-cache' });
    if (!resp.ok) {
        throw new Error(`fetch ${url}: HTTP ${resp.status}`);
    }
    return await resp.json() as BuildManifest;
}

function replayBuildHash(replay: string): string {
    const compact = COMPACT_REPLAY_RE.exec(replay);
    if (compact !== null) {
        return compact[1] ?? '';
    }
    if (HASH_RE.test(replay)) {
        return replay;
    }
    throw new Error('replay= must be an rhrec compact replay or a git hash');
}

async function resolveBuild(): Promise<BuildSelection> {
    const params = new URLSearchParams(window.location.search);
    const replay = params.get('replay');
    if (replay !== null && replay.length > 0) {
        const hash = replayBuildHash(replay);
        return { short: hash, source: 'replay' };
    }

    const latest = await fetchJson(`${WASM_BUILDS_BASE}/latest.json`);
    const short = String(latest.short ?? latest.commit ?? '');
    if (!HASH_RE.test(short)) {
        throw new Error(`latest.json did not contain a valid git hash: ${short}`);
    }
    return { short, source: 'latest' };
}

async function main(): Promise<void> {
    const build = await resolveBuild();
    const buildBase = `${WASM_BUILDS_BASE}/${build.short}`;
    logOk(`[selected ${build.source} build ${build.short}]`);

    logOk('[loading wasm module]');
    const wasm = await import(/* @vite-ignore */ `${buildBase}/robin.js`) as RobinWasmModule;
    await wasm.default(`${buildBase}/robin_bg.wasm`);
    installRpcClient(wasm);

    logOk('[wasm module ready, fetching datadir]');

    const dataUrl = `${BINARIES_BASE}/datadirs/demo-leicester/v3-q80.rhdata.zst`;
    const resp = await fetch(dataUrl, {
        cache: build.source === 'latest' ? 'no-cache' : 'force-cache',
    });
    if (!resp.ok) {
        throw new Error(`fetch ${dataUrl}: HTTP ${resp.status}`);
    }
    const buf = await resp.arrayBuffer();
    logOk(`[datadir fetched: ${buf.byteLength} bytes]`);

    await preloadAssets(wasm, buildBase, build.source === 'latest');

    wasm.wasm_boot(new Uint8Array(buf));
    logOk('[handed off to Rust - winit drives rAF from here]');
}

function installRpcClient(wasm: RobinWasmModule): void {
    if (wasm.rh_rpc_enqueue === undefined) {
        return;
    }
    const encoder = new TextEncoder();
    globalThis.robinRpc = (method: string, params: unknown = null): Promise<unknown> => {
        const id = nextRpcId++;
        const payload = encoder.encode(JSON.stringify({ method, params }));
        return new Promise((resolve, reject) => {
            rpcPending.set(id, { resolve, reject });
            try {
                wasm.rh_rpc_enqueue?.(id, payload);
            } catch (e) {
                rpcPending.delete(id);
                reject(e);
            }
        });
    };
}

async function preloadAssets(
    wasm: RobinWasmModule,
    buildBase: string,
    noCache: boolean,
): Promise<void> {
    if (wasm.wasm_preload_asset === undefined) {
        return;
    }
    const manifestUrl = `${buildBase}/preload-assets.json`;
    const manifestResp = await fetch(manifestUrl, {
        cache: noCache ? 'no-cache' : 'force-cache',
    });
    if (manifestResp.status === 404) {
        return;
    }
    if (!manifestResp.ok) {
        throw new Error(`fetch ${manifestUrl}: HTTP ${manifestResp.status}`);
    }
    const raw = await manifestResp.json() as unknown;
    if (!Array.isArray(raw)) {
        throw new Error(`${manifestUrl} must be a JSON array`);
    }
    for (const entry of raw as PreloadEntry[]) {
        const path = typeof entry === 'string' ? entry : String(entry.path ?? '');
        const url = typeof entry === 'string' ? `${buildBase}/${entry}` : String(entry.url ?? path);
        if (path.length === 0 || url.length === 0) {
            throw new Error(`${manifestUrl} contains an invalid preload entry`);
        }
        const assetUrl = new URL(
            url,
            buildBase.endsWith('/') ? buildBase : `${buildBase}/`,
        ).toString();
        const assetResp = await fetch(assetUrl, {
            cache: noCache ? 'no-cache' : 'force-cache',
        });
        if (!assetResp.ok) {
            throw new Error(`fetch ${assetUrl}: HTTP ${assetResp.status}`);
        }
        const bytes = new Uint8Array(await assetResp.arrayBuffer());
        wasm.wasm_preload_asset(path, bytes);
        logOk(`[preloaded ${path}: ${bytes.byteLength} bytes]`);
    }
}

main().catch((e: unknown) => {
    const msg = e instanceof Error ? e.stack ?? e.message : String(e);
    // eslint-disable-next-line no-console
    console.error(msg);
    logErr(`[boot failed] ${msg}`);
});
