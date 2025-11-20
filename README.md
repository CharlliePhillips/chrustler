# The Chrustler

## Hardware
  1. Brains: Raspbery Pi Zero 2 W
  2. I2S audio card: IQuadIO Codec Zero
  3. Display: SSD1309 I2C (128x64 2.5in OLED)
  4. Time of Flight sensor: ST Micro VL53L1X
  5. MCP23017 GPIO extender
  5. PD Trigger Board
  6. USB-C “flush” mount extension
  7. 2x 3.5mm TRS female jacks
  8. Micro USB OTG cable
  9. 1W 8ohm 3.6cm speaker
  10. Proto-board
  11. Custom PCB for 4x4 Cherry-style mechanical key switches
  12. 16x Otemu Lemon switches
  13. 2x Encoders (from class kit)
  14. 1-2 GPIO duplication module
  15. 8x m2.5 screws & heat set inserts
  16. 12x m3 screws & heat set inserts
  17. 3D printed Chassis
  18. 3D printed faceplate with acrylic window
  19. 2x 3D printed standoffs
  20. 16x 3D printed keycaps
  21. 2x encoder caps
• The parts that you designed or made yourself ^^^
## Libraries
  - Explain Rust stuff
  - The libraries of note:
  1. pitch-detection (https://docs.rs/pitch-detection/latest/pitch_detection/)
  1. Awedio – modified for 64 bit sample indexing (https://docs.rs/awedio/latest/awedio/index.html)
  My fork (https://github.com/CharlliePhillips/vl53l1x-rs)
  2. vl53l1x-rs – modified for compatibility and sensor calibration (https://docs.rs/vl53l1x/latest/vl53l1x/)
  My fork (https://github.com/CharlliePhillips/awedio64)
  3. rppal (https://docs.rs/rppal/latest/rppal/)
  4. mcp23017 (https://docs.rs/mcp23017/latest/mcp23017/)
  5. ssd1306 (https://docs.rs/ssd1306/latest/ssd1306/)
  6. embedded-graphics (https://docs.rs/embedded-graphics/latest/embedded_graphics/)
## Learned Skills
  1. ToF things
  2. Heat set inserts (talk about borked one)
  3. Multi-color 3D printing
  4. Iterating on CAD designs
  5. Linux audio
  6. PCB Fabrication
  7. Not SDs but in class acrylic crazing and I2C speed/capacitance relationship
## Iteration Process
### Initial Testing
  1. Proof of concept program that runs on desktop Linux with audio IO. Included TOF functionality later.
  2. GPIO test program to ensure button & encoders work
  3. Hardware bare on a breadboard
### Ideas That Didn't Work Out
  1. Battery power
  2. Filter frequency sweeping
### Hardware That Had To Be Changed
  1. Keypad
  2. SD Card – OTG adapter
### Lessons Learned
  1. Calibrate your ToF sensor
  2. Consider cable management
  3. Test physical design with small parts before doing a 10 hour 3d print
  4. ALSA(mixer) is fun
  #### Other Potential Improvements
  1. More thorough recording (i.e. adjustable count in, timed recording, input level)
  2. Simple sample trimming
  3. Analog chord keys (with hall effect key switches)
  4. Stereo sound

## The Final Prototype
• Some photos of the physical artifact 
    ◦ Minimum of one overview photo, but feel free to include additional photos in the documentation 
