#!/usr/bin/env node

/**
 * Optimizes all SVG files in `src/lib/icons/svg/` using svgo,
 * then syncs `src/lib/icons/names.ts` via update-icon-names.js.
 *
 * Usage:
 *   pnpm optimize-icons
 */

import { optimize } from "svgo";
import { readFileSync, writeFileSync, readdirSync } from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const uiRoot = path.resolve(__dirname, "..");
const iconsDir = path.join(uiRoot, "src/lib/icons/svg");

/**
 * Custom svgo plugin: replaces any hardcoded color value on fill/stroke
 * attributes with `currentColor`, leaving `none` untouched.
 *
 * @type {import('svgo').CustomPlugin}
 */
const replaceColorsWithCurrentColor = {
	name: "replaceColorsWithCurrentColor",
	fn() {
		return {
			element: {
				enter(node) {
					for (const attr of ["fill", "stroke"]) {
						const value = node.attributes[attr];
						if (value && value !== "none" && value !== "currentColor") {
							node.attributes[attr] = "currentColor";
						}
					}
					if (node.attributes["stroke"] && node.attributes["stroke"] !== "none") {
						node.attributes["vector-effect"] = "non-scaling-stroke";
					}
				},
			},
		};
	},
};

/** @type {import('svgo').Config} */
const svgoConfig = {
	plugins: [
		{
			name: "preset-default",
			params: {
				overrides: {
					// Don't collapse groups — our icons use them intentionally
					collapseGroups: false,
					// Keep viewBox — required for proper scaling
					removeViewBox: false,
				},
			},
		},
		// Ensure width/height are set to 100% so icons scale via CSS
		{
			name: "addWidthHeight100Percent",
			fn() {
				return {
					element: {
						enter(node) {
							if (node.name === "svg") {
								node.attributes.width = "100%";
								node.attributes.height = "100%";
							}
						},
					},
				};
			},
		},
		// Replace any hardcoded colors with currentColor
		replaceColorsWithCurrentColor,
	],
};

function main() {
	const svgFiles = readdirSync(iconsDir).filter((f) => f.endsWith(".svg"));

	let optimized = 0;
	let unchanged = 0;
	let totalSavedBytes = 0;

	for (const file of svgFiles) {
		const filePath = path.join(iconsDir, file);
		const original = readFileSync(filePath, "utf-8");
		const result = optimize(original, { path: filePath, ...svgoConfig });
		const optimizedSvg = result.data;

		if (optimizedSvg !== original) {
			const saved = original.length - optimizedSvg.length;
			totalSavedBytes += saved;
			writeFileSync(filePath, optimizedSvg, "utf-8");
			optimized++;
		} else {
			unchanged++;
		}
	}

	console.warn(`Optimized ${optimized} icon(s), ${unchanged} already optimal.`);
	if (totalSavedBytes > 0) {
		console.warn(`Saved ${(totalSavedBytes / 1024).toFixed(2)} KB total.`);
	}
}

main();
