import tseslint from 'typescript-eslint';
import pluginVue from 'eslint-plugin-vue';
import prettier from 'eslint-config-prettier';

export default tseslint.config(
	{ ignores: ['dist', 'coverage', 'node_modules'] },
	...tseslint.configs.recommended,
	...pluginVue.configs['flat/recommended'],
	{
		files: ['**/*.vue'],
		languageOptions: {
			parserOptions: { parser: tseslint.parser },
		},
	},
	{
		rules: {
			'@typescript-eslint/no-explicit-any': 'error',
		},
	},
	prettier,
);
