desktop_data = configuration_data()
desktop_data.set('APPLICATION_ID', application_id)
desktop_file = i18n.merge_file(
  input: configure_file(
    input: 'com.github.sixpounder.GameOfLife.desktop.in.in',
    output: 'com.github.sixpounder.GameOfLife.desktop.in',
    configuration: desktop_data,
  ),
  output: 'com.github.sixpounder.GameOfLife.desktop',
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(get_option('datadir'), 'applications')
)

desktop_utils = find_program('desktop-file-validate', required: false)
if desktop_utils.found()
  test('Validate desktop file', desktop_utils,
    args: [desktop_file]
  )
endif

appstream_file = i18n.merge_file(
  input: 'com.github.sixpounder.GameOfLife.appdata.xml.in',
  output: 'com.github.sixpounder.GameOfLife.appdata.xml',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(get_option('datadir'), 'appdata')
)

appstream_util = find_program('appstream-util', required: false)
if appstream_util.found()
  test('Validate appstream file', appstream_util,
    args: ['validate', appstream_file]
  )
endif

install_data('com.github.sixpounder.GameOfLife.gschema.xml',
  install_dir: join_paths(get_option('datadir'), 'glib-2.0/schemas')
)

gnome.compile_schemas()

compile_schemas = find_program('glib-compile-schemas', required: true)
if compile_schemas.found()
  test('Validate and compile schema file',
    compile_schemas,
    args: ['--strict', '--dry-run', meson.current_source_dir()],
    suite: ['lint']
  )
endif

subdir('icons')
