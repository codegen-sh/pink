import React from "react";

interface SampleProps {
	title: string;
	description: string;
	isActive: boolean;
	onClick: () => void;
	data: any;
}

export class SamplePresenter {
	constructor(private props: SampleProps) {}

	vm() {
		const { title, description, isActive, onClick, data } = this.props;

		return {
			// Simple repeats of props
			heading: title,
			content: description,
			active: isActive,

			// Non-repeats (transformed props)
			upperTitle: title.toUpperCase(),

			// Repeats of other properties
			displayTitle: "heading",

			// Function handlers
			handleClick: onClick,

			// Complex properties
			formattedData: JSON.stringify(data),
		};
	}

	render() {
		const vm = this.vm();

		return (
			<div className={vm.active ? "active" : ""}>
				<h1>{vm.heading}</h1>
				<p>{vm.content}</p>
				<button onClick={vm.handleClick}>Click me</button>
				<pre>{vm.formattedData}</pre>
			</div>
		);
	}
}
