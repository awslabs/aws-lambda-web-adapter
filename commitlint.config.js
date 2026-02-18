module.exports = {
  extends: ['@commitlint/config-conventional'],
  rules: {
    'type-enum': [
      2,
      'always',
      [
        'feat',
        'fix',
        'docs',
        'example',
        'examples',
        'chore',
        'refactor',
        'perf',
        'test',
        'ci',
        'revert',
      ],
    ],
    'subject-case': [0],
    'body-max-line-length': [0],
    'footer-max-line-length': [0],
    'header-max-length': [2, 'always', 120],
  },
};
