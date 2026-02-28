#!/usr/bin/env node
/**
 * update-file-icons.mjs
 *
 * Reads SVG files from a source directory, optimises them (strips width/height,
 * collapses whitespace, removes clip-path wrappers, replaces hardcoded hex
 * colours with CSS variables), then patches any matching keys inside
 * packages/ui/src/lib/components/file/fileIcons.ts.
 *
 * Usage:
 *   node scripts/update-file-icons.mjs <svg-dir>
 */

import { readFileSync, writeFileSync, readdirSync } from "fs";
import { join, basename } from "path";
import { dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
// scripts/ lives one level inside packages/ui – ROOT points to packages/ui
const ROOT = join(__dirname, "..");

// ── 1. Colour map – hardcoded hex → CSS variable ────────────────────────────
const COLOR_MAP = {
	"#807976": "var(--file-icon-gray)",
	"#F5700B": "var(--file-icon-orange)",
	"#f5700b": "var(--file-icon-orange)",
	"#16A34A": "var(--file-icon-green)",
	"#16a34a": "var(--file-icon-green)",
	"#A855F7": "var(--file-icon-purple)",
	"#a855f7": "var(--file-icon-purple)",
	"#F472B6": "var(--file-icon-pink)",
	"#f472b6": "var(--file-icon-pink)",
	"#0EA5E9": "var(--file-icon-blue)",
	"#0ea5e9": "var(--file-icon-blue)",
	"#EAB308": "var(--file-icon-yellow)",
	"#eab308": "var(--file-icon-yellow)",
	"#EF4444": "var(--file-icon-red)",
	"#ef4444": "var(--file-icon-red)",
	"#60A5FA": "var(--file-icon-blue)",
	"#60a5fa": "var(--file-icon-blue)",
	"#4ADE80": "var(--file-icon-green)",
	"#4ade80": "var(--file-icon-green)",
	"#F87171": "var(--file-icon-red)",
	"#f87171": "var(--file-icon-red)",
	"#F59E0B": "var(--file-icon-yellow)",
	"#f59e0b": "var(--file-icon-yellow)",
	"#3B82F6": "var(--file-icon-blue)",
	"#3b82f6": "var(--file-icon-blue)",
	"#22C55E": "var(--file-icon-green)",
	"#22c55e": "var(--file-icon-green)",
	"#EC4899": "var(--file-icon-pink)",
	"#ec4899": "var(--file-icon-pink)",
	"#8B5CF6": "var(--file-icon-purple)",
	"#8b5cf6": "var(--file-icon-purple)",
};

// ── 2. Optimise a raw SVG string ─────────────────────────────────────────────
function optimizeSvg(raw) {
	let svg = raw;

	// Collapse all whitespace / newlines into single spaces
	svg = svg.replace(/\s+/g, " ").trim();

	// Remove width="14" height="14" from <svg …>
	svg = svg.replace(/\s*width="14"\s*/g, " ").replace(/\s*height="14"\s*/g, " ");

	// Normalise <svg …> attribute order: xmlns first, then fill="none", then viewBox
	svg = svg.replace(/<svg([^>]*)>/, (_, attrs) => {
		const get = (name) => {
			const m = attrs.match(new RegExp(`${name}="([^"]*)"`));
			return m ? m[1] : null;
		};
		const xmlns = get("xmlns") ?? "http://www.w3.org/2000/svg";
		const viewBox = get("viewBox") ?? "0 0 14 14";
		return `<svg xmlns="${xmlns}" fill="none" viewBox="${viewBox}">`;
	});

	// Unwrap <g clip-path="…"> … </g> + remove <defs>…</defs>
	svg = svg.replace(/<g\s+clip-path="[^"]*">\s*/g, "");
	svg = svg.replace(/\s*<\/g>/g, "");
	svg = svg.replace(/\s*<defs>.*?<\/defs>\s*/gs, "");

	// Fix any space left between <svg ...> and first child element
	svg = svg.replace(/(<svg[^>]*>) +</g, "$1<");

	// Replace hardcoded colours
	for (const [hex, cssVar] of Object.entries(COLOR_MAP)) {
		const escaped = hex.replace("#", "\\#");
		svg = svg.replace(new RegExp(escaped, "gi"), cssVar);
	}

	// Tidy up any double spaces left over
	svg = svg.replace(/  +/g, " ").replace(/ >/g, ">").trim();

	return svg;
}

// ── 3. Derive the fileIcons.ts key from the filename ────────────────────────
function keyFromFilename(filename) {
	return basename(filename, ".svg");
}

// ── 4. Patch fileIcons.ts ────────────────────────────────────────────────────
function patchFileIcons(iconsTs, key, newSvg) {
	const escapedKey = key.replace(/[-[\]{}()*+?.,\\^$|#\s]/g, "\\$&");

	// Try quoted key first, then unquoted
	const patterns = [
		new RegExp(`("${escapedKey}":\\s*\n?\\s*)'(<svg[^']*)'`, "s"),
		new RegExp(`(\\b${escapedKey}:\\s*\n?\\s*)'(<svg[^']*)'`, "s"),
	];

	for (const pattern of patterns) {
		if (pattern.test(iconsTs)) {
			return iconsTs.replace(pattern, `$1'${newSvg}'`);
		}
	}

	return null;
}

// ── 5. Main ──────────────────────────────────────────────────────────────────
const svgDir = process.argv[2];
if (!svgDir) {
	console.error("Usage: node scripts/update-file-icons.mjs <svg-dir>");
	process.exit(1);
}

const tsFile = join(ROOT, "src/lib/components/file/fileIcons.ts");
let tsContent = readFileSync(tsFile, "utf8");

const svgFiles = readdirSync(svgDir).filter((f) => f.endsWith(".svg"));

let updated = 0;
let skipped = 0;
const newEntries = [];

for (const file of svgFiles) {
	const key = keyFromFilename(file);
	const raw = readFileSync(join(svgDir, file), "utf8");
	const optimized = optimizeSvg(raw);

	const patched = patchFileIcons(tsContent, key, optimized);

	if (patched) {
		tsContent = patched;
		console.log(`✓ updated  "${key}"`);
		updated++;
	} else {
		console.log(`⚠ no match "${key}" – skipping (add manually if needed)`);
		newEntries.push({ key, svg: optimized });
		skipped++;
	}
}

writeFileSync(tsFile, tsContent, "utf8");

console.log(`\nDone. ${updated} updated, ${skipped} skipped.`);
if (newEntries.length) {
	console.log("\nSVG values for skipped icons (add manually to fileIcons.ts):");
	for (const { key, svg } of newEntries) {
		console.log(`\n  "${key}":\n    '${svg}',`);
	}
}
