#!/bin/bash
# /home/charlie/.config/autostart/chrustler_autostart.sh
# this will not do anything unless called by a .desktop file
aconnect -x
pulseaudio --start
echo '5s'
env sleep 5
/home/charlie/chrustler/target/release/chrustler >> /home/charlie/chrustler.log 2>&1
