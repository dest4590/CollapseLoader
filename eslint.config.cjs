module.exports = [
    {
        ignores: [
            'node_modules/**',
            'dist/**',
            'src-tauri/target/**',
            '**/*.log',
            'public/**',
            '*.local',
        ],
    },

    {
        files: ['**/*.vue'],
        languageOptions: {
            parser: require('vue-eslint-parser'),
            parserOptions: {
                parser: require('@typescript-eslint/parser'),
                ecmaVersion: 2020,
                sourceType: 'module',
                extraFileExtensions: ['.vue'],
            },
        },
        plugins: {
            vue: require('eslint-plugin-vue'),
            '@typescript-eslint': require('@typescript-eslint/eslint-plugin'),
        },
        rules: {
            '@typescript-eslint/no-unused-vars': [
                'error',
                { argsIgnorePattern: '^_' },
            ],
        },
    },

    {
        files: ['**/*.ts', '**/*.tsx', '**/*.js'],
        languageOptions: {
            parser: require('@typescript-eslint/parser'),
            parserOptions: {
                ecmaVersion: 2020,
                sourceType: 'module',
            },
        },
        plugins: {
            '@typescript-eslint': require('@typescript-eslint/eslint-plugin'),
        },
        rules: {
            '@typescript-eslint/no-unused-vars': [
                'error',
                { argsIgnorePattern: '^_' },
            ],
            '@typescript-eslint/explicit-module-boundary-types': 'off',
        },
    },
];
