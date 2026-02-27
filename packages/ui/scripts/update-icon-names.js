#!/usr/bin/env node

/**
 * Syncs names.ts with the actual SVG files in the icons/svg folder.
 *
 * Reads all .svg files from `packages/ui/src/lib/icons/svg/` and regenerates
 * `packages/ui/src/lib/icons/names.ts` to match. Icons that no longer
 * have an SVG file are removed; new SVG files are added.
 *
 * Usage:
 *   pnpm update-icons
 */

import { readFileSync, writeFileSync, readdirSync } from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const uiRoot = path.resolve(__dirname, "..");
const iconsDir = path.join(uiRoot, "src/lib/icons/svg");
const iconNamesFile = path.join(uiRoot, "src/lib/icons/names.ts");

function main() {
	// Read all .svg files from the icons directory
	const svgFiles = readdirSync(iconsDir)
		.filter((f) => f.endsWith(".svg"))
		.map((f) => f.replace(".svg", ""))
		.sort();

	// Read current names to diff
	let currentNames = [];
	try {
		const content = readFileSync(iconNamesFile, "utf-8");
		const match = content.match(/\[([^\]]+)\]/s);
		if (match) {
			currentNames = [...match[1].matchAll(/"([^"]+)"/g)].map((m) => m[1]);
		}
	} catch {
		// File doesn't exist yet
	}

	const svgSet = new Set(svgFiles);
	const currentSet = new Set(currentNames);
	const removed = currentNames.filter((name) => !svgSet.has(name));
	const added = svgFiles.filter((name) => !currentSet.has(name));

	// Generate new file content
	const iconEntries = svgFiles.map((name) => `\t"${name}"`).join(",\n");
	const newContent = `/**
 * Auto-generated icon name list from \`src/lib/icons/svg/*.svg\`.
 * Run \`pnpm update-icons\` to regenerate.
 */
export const newIconNames = [
${iconEntries},
] as const;

export type NewIconName = (typeof newIconNames)[number];
`;

	const existingContent = (() => {
		try {
			return readFileSync(iconNamesFile, "utf-8");
		} catch {
			return "";
		}
	})();

	if (existingContent === newContent) {
		console.warn("Already in sync — no changes needed.");
		return;
	}

	writeFileSync(iconNamesFile, newContent, "utf-8");

	if (added.length > 0) {
		console.warn("Added:");
		for (const name of added) {
			console.warn(`  + ${name}`);
		}
	}
	if (removed.length > 0) {
		console.warn("Removed:");
		for (const name of removed) {
			console.warn(`  - ${name}`);
		}
	}

	console.warn(`\nSynced ${svgFiles.length} icons → ${path.relative(uiRoot, iconNamesFile)}`);
}

main();
