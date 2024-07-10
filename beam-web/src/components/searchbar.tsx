import { Input } from "@/components/ui/input";
import { SVGProps } from "react";
import { JSX } from "react/jsx-runtime";

export const SearchBar = () => {
	return (
		<>
			<div className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground">
				<SearchIcon className="h-4 w-4" />
			</div>
			<Input
				type="search"
				placeholder="Search videos..."
				className="w-full rounded-full bg-muted pl-8 pr-4 py-2 text-sm"
			/>
		</>
	);
};

const SearchIcon = (props: JSX.IntrinsicAttributes & SVGProps<SVGSVGElement>) => {
	return (
		<svg
            {...props}
			xmlns="http://www.w3.org/2000/svg"
			width="24"
			height="24"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			strokeWidth="2"
			strokeLinecap="round"
			strokeLinejoin="round"
		>
            <title>Search</title>
			<circle cx="11" cy="11" r="8" />
			<path d="m21 21-4.3-4.3" />
		</svg>
	);
}; // TODO: Replace search icon with one from icon pack
