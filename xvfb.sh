#!/bin/bash

export DISPLAY=:1
function keepUpScreen() {
  echo "running keepUpScreen()"
  while true; do
        sleep .25
        if [ -z $(pidof Xvfb) ]; then
                Xvfb $DISPLAY -screen $DISPLAY 1280x1024x16 &
        fi;
  done;
}


keepUpScreen &
echo "running: ${@}"
exec "$@"