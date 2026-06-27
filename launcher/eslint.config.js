import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import tseslint from 'typescript-eslint'
import prettier from 'eslint-config-prettier'

export default tseslint.config(
  { ignores: ['dist', 'src-tauri', 'node_modules'] },
  {
    extends: [js.configs.recommended, ...tseslint.configs.strict],
    files: ['**/*.{ts,tsx}'],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
    },
    plugins: {
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
      // invoke() — это внешняя система (Tauri backend), вызов setState после него — корректный паттерн
      'react-hooks/set-state-in-effect': 'off',

      // Запрет any — используй unknown или конкретные типы
      '@typescript-eslint/no-explicit-any': 'error',
      // Явный импорт типов — не смешивай runtime и type imports
      '@typescript-eslint/consistent-type-imports': ['error', { prefer: 'type-imports' }],
      // Non-null assertion — предпочитай явные проверки
      '@typescript-eslint/no-non-null-assertion': 'warn',
      // Запрет пустых функций без причины
      '@typescript-eslint/no-empty-function': 'warn',

      // console.log в продакшне — только warn/error допустимы
      'no-console': ['warn', { allow: ['warn', 'error'] }],
      // Всегда const, если переменная не переприсваивается
      'prefer-const': 'error',
      // Нет дублирующих импортов
      'no-duplicate-imports': 'error',
    },
  },
  prettier,
)
