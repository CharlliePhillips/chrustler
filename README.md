# The Chrustler
  Chrustler is a combination of the words 'chord' (what the instrument plays) 'Rust' (the language the software is written in) and 'sampler' (the family of instruments it belongs to). This instrument takes in samples recorded using the device or loaded from a USB drive and detects their frequency. Then the user can play that sample as chords in one of the western music scales. This is my final project for CS3651 at Georgia Tech.
## Hardware
  1. Brains: Raspbery Pi Zero 2 W  
    - Runs the software itself.
  2. I2S audio card: IQuadIO Codec Zero  
    - Used for internal mic and speaker, AUX in and out headers.
    - Also has a 5 band EQ used for the high and low pass filters.
  3. Display: SSD1309 I2C (128x64 2.5in OLED)  
    - Displays current playback information.
  4. Time of Flight sensor: ST Micro VL53L1X  
    - Has 2 regions of interest where the right side controls the level of the high frequencies (low pass filter), and the left side controls the level of the low frequencies (high pass filter)
  5. MCP23017 GPIO extender  
    - Provides additional GPIO for keypad input.
  5. PD Trigger Board  
    - Takes USB-C PD and turns it into 5V to power everything.
  6. USB-C “flush” mount extension  
    - This is intended for installation in a car to add a USB-C port, but provides a way to panel mount the power input (and USB-C is better than micro-B on the pi).
  7. 2x 3.5mm TRS female jacks  
    - Connected to the AUX in and out headers on the Codec Zero for external audio IO.
  8. Micro USB OTG cable  
    - Used to expose a full size USB port for portable loading and storing of sample files.
  9. 1W 8ohm 3.6cm speaker  
    - Connected to the Codec Zero as the internal speaker.
  10. Proto-board  
    - Used for a 5V and ground buses as well as I2C data and clock buses.
  11. Custom PCB for 4x4 Cherry-style mechanical key switches  
    - Designed in KiCAD and manufactured at the GT Hive to hold 16 cherry-style switches and the MCP23017.  
    - The KiCAD project and board drawings are in [/keypad_pcb/](/keypad_pcb/)  
  12. 16x Otemu Lemon switches  
    - Spare linear Cherry-style switches I had that felt nice for the primary input method.  
  13. 2x Addicore AD267 Rotary Encoders (from class kit)  
    - Included in parts kit for class and used for volume adjustment, IO change, key change, and sample selection.  
  14. 1-2 GPIO duplication module  
    - Gives one set of 40 GPIO pins to connect to the Codec Zero, and another for the other GPIO devices, I2C, and power.  
  15. 8x m2.5 screws & heat set inserts
  16. 12x m3 screws & heat set inserts
  17. 3D printed Chassis  
    - Housing with standoffs to add heat-set inserts, and mount everything nicely. Designed in OpenSCAD.
  18. 3D printed faceplate with acrylic window  
    - Covers the main chassis and has inset text to identify IO ports, Encoder functions, and internal microphone location. Designed in OpenSCAD.  
    - The file for the chassis and faceplate is at [scad/chrustler chassis.scad](scad/crustler%20chassis.scad).  
  19. 2x 3D printed standoffs  
    - used to support the Codec Zero on top of the Pi accounting for the height added by the GPIO duplicator. Designed in OpenSCAD.  
    - File is at [scad/soundcard standoff.scad](scad/soundcard%20standoff.scad)  
  20. 16x 3D printed keycaps  
    - Custom 3D printed keycaps with the appropriate legends built from the [keycap playground OpenSCAD project]().  
    - My modified version of the keycap plaground file is at [/scad/keycap_playground/keycap_playground.scad](/scad/keycap_playground/keycap_playground.scad). This was used to make a base stl file, and then imported to [/scad/legend_caps.scad](/scad/legend_caps.scad) to add the legends.
  21. 2x encoder caps  
    - Spares I had from an SP404-OG after replacing them on that device.

## Libraries
  I wanted to write the software in Rust from the start and it provides a nice way to handle interrupts/multithreading, errors, and has pretty good library support on Raspberry Pi devices. I used Rust's standard library, and a couple of other common ones such as `serde` for storing the time of flight sensor's calibration data. The libraries of note that are used to interact with the hardware are:
  1. [pitch-detection](https://docs.rs/pitch-detection/latest/pitch_detection/)  
    - Provides a function for detecting the audio frequency on an array of individual sample points from an audio file.
  1. [awedio](https://docs.rs/awedio/latest/awedio/index.html)  
    - This library handles the playback of samples and provides functions to adjust the speed of the sample (very important for pitch correction) as well as a controller to stop/gate the playing samples.  
    - During development I kept running into an integer overflow issue when playing long samples. To fix this I changed a bunch of the related variables from unsigned 32 bit to unsigned 64 bit numbers.  
    - [My fork](https://github.com/CharlliePhillips/awedio64)
  2. [vl53l1x-rs](https://docs.rs/vl53l1x/latest/vl53l1x/)  
    - This library provides functions for the time of flight sensor.  
    - This nor any other 3rd party library I could find supported all five calibration functions, and so I added wrappers to the ST Micro API for them.  
    - Additionally the `usleep()` function in the ST Micro library this crate wraps wasn't compiling on the Pi, so I changed it to `nanosleep()`.  
    - [My fork](https://github.com/CharlliePhillips/vl53l1x-rs)
  3. [rppal](https://docs.rs/rppal/latest/rppal/)  
    - Provides functions to access the GPIO and I2C hardware on the Pi.
  4. [mcp23017](https://docs.rs/mcp23017/latest/mcp23017/)  
    - Provides functions to interface with GPIO on the MCP23017.
  5. [ssd1306](https://docs.rs/ssd1306/latest/ssd1306/)  
    - Provides structures and functions to interface with an SSD1306 display.  
    - The SSD1306 display is identical to the SSD1309 display used for this project (aside from the physical size, and default address), and by changing the I2C address provided during initialization is cross-compatible.
  6. [embedded-graphics](https://docs.rs/embedded-graphics/latest/embedded_graphics/)  
    - Provides structures and functions for creating the text shown on the display.  

  - The libraries' documentation is linked where their git repos can also be found.
## Learned Skills
  1. Time of Flight Sensors  
  - Before taking on this project I was totally unaware that this type of distance sensor existed. I had originally intended to use an ultrasonic sensor, or a more traditional IR sensor.
  - I picked the model of ToF sensor I did because of it's existing Rust library support, and the user programmable Regions of interest. Those regions of interest will still pick up on objects outside the region of interest if they cover enough of the sensor's field of view but work reasonably well with two hands at different distances.
  - It is pretty easy to interface with the sensor over I2C but they do not come calibrated from the factory and must go through a 3-step calibration process in order to get accurate readings that properly distinguish between the regions of interest.
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
