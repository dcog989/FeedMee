import eslintPluginSvelte from 'eslint-plugin-svelte';
import tseslint from '@typescript-eslint/eslint-plugin';
import tsParser from '@typescript-eslint/parser';
import svelteParser from 'svelte-eslint-parser';
import globals from 'globals';

/** @type {import('eslint').Linter.FlatConfig[]} */
export default [
    {
        ignores: [
            'build/**',
            '.svelte-kit/**',
            'dist/**',
            'node_modules/**',
            'src-tauri/**',
            'scripts/**',
        ],
    },
    {
        files: ['**/*.ts'],
        languageOptions: {
            parser: tsParser,
            parserOptions: { project: './tsconfig.json' },
            globals: { ...globals.browser },
        },
        plugins: { '@typescript-eslint': tseslint },
        rules: {
            ...tseslint.configs['recommended'].rules,
            '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
            '@typescript-eslint/no-explicit-any': 'warn',
        },
    },
    {
        files: ['**/*.svelte'],
        languageOptions: {
            parser: svelteParser,
            parserOptions: { parser: tsParser },
            globals: { ...globals.browser },
        },
        plugins: { svelte: eslintPluginSvelte, '@typescript-eslint': tseslint },
        rules: {
            ...eslintPluginSvelte.configs['flat/recommended'][1]?.rules,
            ...tseslint.configs['recommended'].rules,
            '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
            '@typescript-eslint/no-explicit-any': 'warn',
        },
    },
];
