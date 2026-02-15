#set -e

function vic_run_func() {
  $BINARY $@
}

function vic_build_func_internal() {
  vic_build_func
  exit_code=$?
  if [[ "$exit_code" != "0" ]]
  then
    echo "vic wrapper: BUILD FAILED, not running"
    return $exit_code
  fi
}

function vic_wrapper_run() {

  dev_null=$(command -v vic_main ) && vic_main "$@"

  if [ ! -z ${VIC_NO_RUN+x} ]
  then
    return
  fi

  if [[ "$1" == "b" ]]
  then
    vic_build_func_internal && vic_run_func "${@:2}"

  # build only
  elif [[ "$1" == "bo" ]]
  then
    vic_build_func_internal

  # run only
  elif [[ "$1" == "ro" ]]
  then
    vic_run_func "${@:2}"

  # if VIC_WRAPPER_REBUILD is set, then build as well
  elif [ ! -z ${VIC_WRAPPER_REBUILD+x} ]
  then
    vic_build_func_internal && vic_run_func "$@"

  # default only run
  else
    vic_run_func "$@"

  fi

}


