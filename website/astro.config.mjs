// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	site: 'https://traitclaw.github.io',
	base: '/traitclaw',
	integrations: [
		starlight({
			title: 'TraitClaw',
			description: 'A Rust AI Agent Framework — Simple by default, powerful when needed.',
			logo: {
				dark: './src/assets/logo-dark.svg',
				light: './src/assets/logo-light.svg',
				replacesTitle: false,
			},
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/traitclaw/traitclaw' },
			],
			editLink: {
				baseUrl: 'https://github.com/traitclaw/traitclaw/edit/main/website/',
			},
			customCss: ['./src/styles/custom.css'],
			sidebar: [
				{
					label: 'Getting Started',
					items: [
						{ label: 'Introduction', slug: 'getting-started/introduction' },
						{ label: 'Installation', slug: 'getting-started/installation' },
						{ label: 'Quick Start', slug: 'getting-started/quickstart' },
					],
				},
				{
					label: 'Core Concepts',
					items: [
						{ label: 'Architecture', slug: 'concepts/architecture' },
						{ label: 'Providers', slug: 'concepts/providers' },
						{ label: 'Tools', slug: 'concepts/tools' },
						{ label: 'Memory', slug: 'concepts/memory' },
						{ label: 'Streaming', slug: 'concepts/streaming' },
						{ label: 'Steering', slug: 'concepts/steering' },
					],
				},
				{
					label: 'Guides',
					items: [
						{ label: 'Custom Provider', slug: 'guides/custom-provider' },
						{ label: 'Reasoning Strategies', slug: 'guides/reasoning-strategies' },
						{ label: 'Multi-Agent Teams', slug: 'guides/multi-agent' },
						{ label: 'RAG Pipeline', slug: 'guides/rag-pipeline' },
						{ label: 'MCP Integration', slug: 'guides/mcp-integration' },
						{ label: 'Observability', slug: 'guides/observability' },
					],
				},
				{
					label: 'Why TraitClaw?',
					items: [
						{ label: 'vs TypeScript Frameworks', slug: 'comparisons/vs-typescript' },
						{ label: 'Why Rust for AI Agents?', slug: 'comparisons/why-rust' },
					],
				},
				{
					label: 'API Reference',
					autogenerate: { directory: 'reference' },
				},
			],
		}),
	],
});
