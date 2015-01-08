{
  'variables': {
    'rustc': 'rustc'
  },
  'targets': [
    {
      'target_name': 'node-codius-sandbox',
      'sources': [ 'src/node-module.cpp'],
      'libraries': [
        '../target/libcodius-sandbox-core-0352c0c5f8362e15.a',
        '-lseccomp'
      ],
      'cflags': ['-std=c++11']
    }
  ]
}
