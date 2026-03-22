import js from '@eslint/js';
import globals from 'globals';

export default [
  js.configs.recommended,
  {
    files: ['jsmodules/**/*.js'],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: 'module',
      globals: {
        ...globals.browser,
        vis: 'readonly',    // vis-network loaded via CDN <script>
        THREE: 'readonly',  // three.js loaded via CDN <script>
      },
    },
    rules: {
      'no-unused-vars': ['warn', { varsIgnorePattern: '^_' }],
      'no-console': 'warn',
    },
  },
];
