/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{astro,html,js,jsx,md,mdx,svelte,ts,tsx,vue}'],
	theme: {
		extend: {},
	},
	plugins: [
		require('tailwindcss-animate'),
		require('@vidstack/react/tailwind.cjs')({
		  prefix: 'media',
		}),
		customVariants,
	  ],
}

// TODO: Borrowed from Vidstack player example. See if it could be removed.
function customVariants({ addVariant, matchVariant }) {
	// Strict version of `.group` to help with nesting.
	matchVariant('parent-data', (value) => `.parent[data-${value}] > &`);
  
	addVariant('hocus', ['&:hover', '&:focus-visible']);
	addVariant('group-hocus', ['.group:hover &', '.group:focus-visible &']);
  }
  