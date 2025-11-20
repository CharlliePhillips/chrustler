#!/bin/bash
# /home/charlie/.config/autostart/chrustler_autostart.sh
# this will not do anything unless called by a .desktop file
aconnect -x
pulseaudio --start
echo '20s'
env sleep 10
echo '10s'
env sleep 10
/home/charlie/chrustler/target/release/chrustler >> /home/charlie/chrustler.log 2>&1
