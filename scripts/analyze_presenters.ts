#!/usr/bin/env ts-node

/**
 * Presenter VM Property Analyzer
 *
 * This script analyzes presenter files (*.presenter.tsx or *.presenter.ts) to identify
 * properties in the VM method that are simple repeats of other properties or props.
 *
 * Usage:
 *   npx ts-node scripts/analyze_presenters.ts [directory]
 *
 * If no directory is specified, it will search in the current directory.
 */

import * as fs from "fs";
import * as path from "path";
import * as glob from "glob";
import * as ts from "typescript";

interface PropertyMapping {
	name: string;
	value: string;
	location: {
		file: string;
		line: number;
		column: number;
	};
	isRepeat: boolean;
	repeatedFrom?: string;
}

function findPresenterFiles(directory: string): string[] {
	return glob.sync(`${directory}/**/*.presenter.{ts,tsx}`);
}

function analyzeVmMethod(
	sourceFile: ts.SourceFile,
	filePath: string,
): PropertyMapping[] {
	const propertyMappings: PropertyMapping[] = [];
	const propNames = new Set<string>();

	// First pass: collect all prop names
	function collectProps(node: ts.Node) {
		if (
			ts.isParameter(node) &&
			node.name &&
			ts.isObjectBindingPattern(node.name)
		) {
			node.name.elements.forEach((element) => {
				if (ts.isBindingElement(element) && ts.isIdentifier(element.name)) {
					propNames.add(element.name.text);
				}
			});
		}
		ts.forEachChild(node, collectProps);
	}

	// Second pass: analyze VM method and find property mappings
	function visit(node: ts.Node) {
		if (
			ts.isMethodDeclaration(node) &&
			ts.isIdentifier(node.name) &&
			node.name.text === "vm"
		) {
			// Found VM method
			if (node.body) {
				ts.forEachChild(node.body, findReturnStatement);
			}
		} else {
			ts.forEachChild(node, visit);
		}
	}

	function findReturnStatement(node: ts.Node) {
		if (
			ts.isReturnStatement(node) &&
			node.expression &&
			ts.isObjectLiteralExpression(node.expression)
		) {
			analyzeReturnedObject(node.expression);
		} else {
			ts.forEachChild(node, findReturnStatement);
		}
	}

	function analyzeReturnedObject(objectLiteral: ts.ObjectLiteralExpression) {
		const properties = objectLiteral.properties;
		const valueMap = new Map<string, string>();

		properties.forEach((prop) => {
			if (ts.isPropertyAssignment(prop) && ts.isIdentifier(prop.name)) {
				const propName = prop.name.text;
				const propValue = prop.initializer.getText();
				const { line, character } = sourceFile.getLineAndCharacterOfPosition(
					prop.getStart(),
				);

				valueMap.set(propName, propValue);

				const mapping: PropertyMapping = {
					name: propName,
					value: propValue,
					location: {
						file: filePath,
						line: line + 1,
						column: character + 1,
					},
					isRepeat: false,
				};

				// Check if this property is a simple repeat of a prop
				if (propNames.has(propValue)) {
					mapping.isRepeat = true;
					mapping.repeatedFrom = propValue;
				}

				propertyMappings.push(mapping);
			}
		});

		// Second pass to find properties that repeat other properties
		propertyMappings.forEach((mapping) => {
			if (!mapping.isRepeat && valueMap.has(mapping.value)) {
				mapping.isRepeat = true;
				mapping.repeatedFrom = mapping.value;
			}
		});
	}

	// Start analysis
	collectProps(sourceFile);
	visit(sourceFile);

	return propertyMappings;
}

function analyzeFile(filePath: string): PropertyMapping[] {
	const fileContent = fs.readFileSync(filePath, "utf-8");
	const sourceFile = ts.createSourceFile(
		filePath,
		fileContent,
		ts.ScriptTarget.Latest,
		true,
	);

	return analyzeVmMethod(sourceFile, filePath);
}

function formatResults(
	results: { file: string; properties: PropertyMapping[] }[],
): string {
	let output = "";

	results.forEach((result) => {
		const repeatedProps = result.properties.filter((prop) => prop.isRepeat);

		if (repeatedProps.length > 0) {
			output += `\nFile: ${result.file}\n`;
			output += "=".repeat(result.file.length + 6) + "\n";

			repeatedProps.forEach((prop) => {
				output += `  - ${prop.name}: ${prop.value} (line ${prop.location.line})\n`;
				output += `    Repeats: ${prop.repeatedFrom}\n`;
			});
		}
	});

	if (output === "") {
		output = "No repeated properties found in any presenter files.";
	}

	return output;
}

function main() {
	const directory = process.argv[2] || ".";
	console.log(`Analyzing presenter files in: ${directory}`);

	const presenterFiles = findPresenterFiles(directory);

	if (presenterFiles.length === 0) {
		console.log("No presenter files found.");
		return;
	}

	console.log(`Found ${presenterFiles.length} presenter files.`);

	const results = presenterFiles.map((file) => {
		const properties = analyzeFile(file);
		return { file, properties };
	});

	const formattedResults = formatResults(results);
	console.log("\nResults:");
	console.log(formattedResults);

	// Count statistics
	const totalRepeatedProps = results.reduce(
		(sum, result) =>
			sum + result.properties.filter((prop) => prop.isRepeat).length,
		0,
	);

	const filesWithRepeats = results.filter((result) =>
		result.properties.some((prop) => prop.isRepeat),
	).length;

	console.log("\nSummary:");
	console.log(`Total presenter files analyzed: ${presenterFiles.length}`);
	console.log(`Files with repeated properties: ${filesWithRepeats}`);
	console.log(`Total repeated properties found: ${totalRepeatedProps}`);
}

main();
