// Stub Node.js types so tsc doesn't complain about missing @types/node
declare namespace NodeJS {
	interface ProcessEnv {
		[key: string]: string | undefined;
	}
}
declare const process: {
	env: NodeJS.ProcessEnv;
	argv: string[];
	exit: (code: number) => never;
	cwd: () => string;
};
declare const __dirname: string;
declare function require(module: string): Record<string, unknown>;
declare module "child_process" {
	export function spawn(
		cmd: string,
		args?: string[],
		opts?: Record<string, unknown>,
	): {
		stdout: { on: Function };
		stderr: { on: Function };
		on: Function;
		kill: () => void;
	};
}
declare module "fs" {
	export function readFileSync(path: string, enc?: string): string;
	export function accessSync(path: string): void;
	export function mkdirSync(path: string, opts?: { recursive?: boolean }): void;
	export function writeFileSync(path: string, data: string): void;
	const rmSync: (path: string) => void;
}
