project('game-of-life', 'rust', 
          version: '0.1.0',
    meson_version: '>= 0.59.0',
  default_options: [ 'warning_level=2',
                     'werror=false',
                   ],
)

i18n = import('i18n')

gnome = import('gnome')


cargo_sources = files(
  'Cargo.toml',
  'Cargo.lock',
)

if get_option('profile') == 'development'
  profile = '.Devel'
  if fs.is_dir('.git')
    vcs_tag = run_command('git', 'rev-parse', '--short', 'HEAD').stdout().strip()
    if vcs_tag == ''
      version_suffix = '-devel'
    else
      version_suffix = '-@0@'.format(vcs_tag)
    endif
  else
    version_suffix = '-devel'
  endif
else
  profile = ''
  version_suffix = ''
endif

application_id = 'com.github.sixpounder.GameOfLife@0@'.format(profile)

subdir('data')
subdir('src')
subdir('po')

gnome.post_install(
  glib_compile_schemas: true,
  gtk_update_icon_cache: true,
  update_desktop_database: true,
)
