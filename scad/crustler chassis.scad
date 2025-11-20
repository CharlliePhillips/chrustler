$fn = 64;
include<BOSL2/std.scad>

//CHASSIS
union() {
difference() {
    translate([0,0,22.5])
    cuboid([200,150,45], rounding = 5, except = [TOP,BOT] );
    
    translate([0,0,28.5])
        cube([185, 135, 45], center=true);
    
    translate([93.5,-1.5,28.5])
        cube([7, 100, 39], center=true);
    
    translate([18,-69,30])
        cube([80,5,45], center=true);
    
    {//bus support heatset
        translate([-12.5,-48.5,4.5])
        cylinder(h=6, r=2.4,center=true);
        
        translate([52.5,-48.5,4.5])
        cylinder(h=6, r=2.4,center=true);
    }
    
    {//PD support heatset
        translate([-57.25,20,4.5])
        cylinder(h=6.1, r=1.6,center=true);
        
        translate([-42.75,20,4.5])
        cylinder(h=6.1, r=1.6,center=true);
    }
    
    {//PI support heatset
        translate([-21,25.5,4.5])
            cylinder(h=6.1, r=1.6,center=true);
        
        translate([-21,-32.5,4.5])
            cylinder(h=6.1, r=1.6,center=true);
    }
    
    { // heatset lid
        translate([95.5,70.5,42])
            cylinder(h=6.5, r = 2.4, center=true);
        translate([-95.5,70.5,42])
            cylinder(h=6.5, r = 2.4, center=true);
        translate([95.5,-70.5,42])
            cylinder(h=6.5, r = 2.4, center=true);
        translate([-95.5,-70.5,42])
            cylinder(h=6.5, r = 2.4, center=true);
    }
    
    translate([-96.25,-55,25]) { // PWR jack
        difference() {
            rotate([0,90,0])
                cylinder(h=9, r = 8, center=true);
            
            translate([0,8.125,0])
                cube([9,1,5],  center = true);
        }
    }
    
    { //HP jack
        translate([-96.25,60,25])
        rotate([0,90,0])
            cylinder(h=9, r = 4.75, center=true);
        
        //translate([-94.8,55,25])
        translate([-95.05,60,25])
        rotate([0,90,0])
            cylinder(h=7.5, r = 5.875, center=true);
    }
    
        { //in jack
        translate([-96.25,45,25])
        rotate([0,90,0])
            cylinder(h=9, r = 4.75, center=true);
        
        //translate([-94.8,55,25])
        translate([-95.05,45,25])
        rotate([0,90,0])
            cylinder(h=7.5, r = 5.875, center=true);
    }
    
    { // USB jack
        translate([75, -71.25, 11.25])
            cube([14.5, 8, 7.5], center=true);
        
        translate([75, -69, 11.75])
            cube([18.5, 7.5, 11.5], center=true);
    }
}
    
    {//USB restraint
        translate([84,-44,9.5])
            cube([4, 4, 8],center=true);
        
        translate([66,-44,9.5])
            cube([4, 4, 8],center=true);
    }
    
    difference() {
    translate([-10,48,6]) difference() { // speaker support
        cylinder(h=36.7,r=18,center=false);
        
        cylinder(h=37,r=17, center=false);
        translate([0,-18,19])
        cube([12,4,37],center = true);
    }
        translate([48, -1.75, 40])
        cube([94,94,6], center=true);
    }
    translate([51,-1.75,6]) { // keys support
        difference() {
            translate([38,40,0])
            cylinder(h=30, r=3.5,center=false);
            
            translate([38,40,24])
            cylinder(h=6.1, r=2.4,center=false);
        }
        difference() {
            translate([38,-37,0])
            cylinder(h=30, r=3.5,center=false);
            
            translate([38,-37,24])
            cylinder(h=6.1, r=2.4,center=false);
        }
        difference() {
            translate([-39,40,0])
            cylinder(h=30, r=3.5,center=false);
            
            translate([-39,40,24])
            cylinder(h=6.1, r=2.4,center=false);
        }
        difference() {
            translate([-39,-37,0])
            cylinder(h=30, r=3.5,center=false);
            
            translate([-39,-37,24])
            cylinder(h=6.1, r=2.4,center=false);
        }
        difference() {
            translate([-1,2,0])
            cylinder(h=30, r=2.75,center=false);
            
            //translate([-1,2,28])
            //cylinder(h=4.1, r=1.98,center=false);
        }
        translate([-1,2,28])
            cylinder(h=8, r = 0.8, center=true); 
    }
    
    translate([20,-48.5,0]) { // Bus board support
        translate([-32.5,0,6])
        difference() {
                cylinder(h=3, r=3.5,center=true);
                cylinder(h=6, r=2.4,center=true);
        }
        
        translate([32.5,0,6])
        difference() {
            cylinder(h=3, r=3.5,center=true);
            cylinder(h=6, r = 2.4, center=true);
        }
     }
     
        translate([-50,20,0]) { // PD board support
            translate([-7.25,0,6])
            difference() {
                cylinder(h=3, r=2.75,center=true);
                cylinder(h=6.1, r=1.6,center=true);
            }
            translate([7.25,0,6])
            difference() {
                cylinder(h=3, r=2.75,center=true);
                cylinder(h=6.1, r=1.6,center=true);
            }
            translate([0,34,6])
                cube([14,2,3], center=true);

     }
    
    translate([0,3.5,0]){ // pi support
    translate([2,22,6])  {
        cylinder(h=.5, r=2.75,center=true);
            translate([0,0,2])
            cylinder(h=4, r = 0.8, center=true);
    }
    translate([2,-36,6])  {// pi support
        cylinder(h=.5, r=2.75,center=true);
            translate([0,0,2])
            cylinder(h=4, r = 0.8, center=true);
    }
    translate([-21,22,6])  {
        difference() {
            cylinder(h=.5, r=2.75,center=true);
            //translate([0,0,2])
            //cylinder(h=4, r = 1.175, center=true);
            cylinder(h=6.1, r=1.6,center=true);
        }
    }
    translate([-21,-36,6])  {
        difference() {
            cylinder(h=.5, r=2.75,center=true);
            //translate([0,0,2])
            //cylinder(h=4, r = 1.175, center=true
            cylinder(h=6.1, r=1.6,center=true);
        }
    }

    }
    translate([-50, 0,6]) { // display support
        difference() {
            translate([21.5,32.5,0])
            cylinder(h=33, r=2.75,center=false);
            
            //translate([21.5,32.5,29])
            //cylinder(h=4.1, r=1.98,center=false);
        }
        translate([21.5,32.5,30.5])
            cylinder(h=8, r = 1.175, center=true); 
        
        difference() {
            translate([-21.5,32.5,0])
            cylinder(h=33, r=2.75,center=false);
            
            translate([-21.5,32.5,27])
            cylinder(h=6.1, r=1.6,center=false);
        }
        
        difference() {
            translate([21.5,-32.5,0])
            cylinder(h=33, r=2.75,center=false);
            
            translate([21.5,-32.5,27])
            cylinder(h=6.1, r=1.6,center=false);
        }
        
        difference() {
            translate([-21.5,-32.5,0])
            cylinder(h=33, r=2.75,center=false);
            
            //translate([-21.5,-32.5,29])
            //cylinder(h=4.1, r=1.98,center=false);
        }
        translate([-21.5,-32.5,30.5])
            cylinder(h=8, r = 1.175, center=true); 
    }
    
    
        translate([-84, 0,6]) { // tof support
        difference() {
            translate([0,10,0])
            cylinder(h=36, r=3.5,center=false);
            
            translate([0,10,30])
            cylinder(h=6.1, r=2.4,center=false);
        }
        difference() {
            translate([0,-10,0])
            cylinder(h=36, r=3.5,center=false);
            
            translate([0,-10,30])
            cylinder(h=6.1, r=2.4,center=false);
        }
    }
}
// M2.5 holes
// cylinder(h=4, radius = 1.98, center=true);

//M3 holes
// cylinder(h=4, radius = 2.4, center=true);

translate([0,0,146.5]) { // LID 
    //translate([0,0,23.25])
    
difference() {
    translate([0,0,-0.5])
    cuboid([200,150,2], rounding = 5, except = [TOP,BOT] );
    
    translate([2,-26.5,0])
    cylinder(h=4, r=1, center=true);
    
    { // TOF cavity
        translate([-82.5,0,0])
        cuboid([5.75,10.5,4], rounding = 2.875, except = [TOP,BOT]);
    }
    
    { // display cavity
        translate([-50,0,0])
            cube([48,70.2,4], center=true);
        translate([-50.5,-.5,-1.025])
            cube([52,74.2,1], center=true);
    }
    
    translate([0,0,-0.25]) { // pot text inset
        translate([-25,-54,.6])
        rotate([0,0,90])
        linear_extrude(.45,center=true)
        text("VOL/IO", size = 6, halign = "center", valign= "center");
        
        translate([25,-57,.6])
        rotate([0,0,90])
        linear_extrude(.45,center=true)
        text("KEY/SND", size = 6, halign = "center", valign= "center");
        
        translate([2,-16,.6])
        rotate([0,0,90])
        linear_extrude(.45,center=true)
        text("MIC", size = 6, halign = "center", valign= "center");
        
        translate([-92,0,.6])
        rotate([0,0,90])
        linear_extrude(.395,center=true)
        text("TOF", size = 6, halign = "center", valign= "center");

        translate([-88,46,.6])
        rotate([0,0,90])
        linear_extrude(.395,center=true)
        text("EXT: IN | OUT", size = 6, halign = "center", valign= "center");
        
        translate([-88,-54,.6])
        rotate([0,0,90])
        linear_extrude(.395,center=true)
        text("PWR", size = 6, halign = "center", valign= "center");
    }
    
    //MCP topside cutout
    //translate([72.5,-52.5,-.51])
    //cube([30,30,.5], center=true);
    
    { // pot holes
         translate([-10,-55,0])
            cylinder(h=4, r = 3.5, center=true);
        
        translate([10,-55,0])
            cylinder(h=4, r = 3.5, center=true);
    }
    translate([-10,48,-.5]) { // speaker grill
        difference() {
            cylinder(h = 2.5, r = 18, center=true);
            
            rotate([0,0,45])
            translate([0,-16,0])
                cube([19.5, 2,2.5], center = true);
            
            rotate([0,0,45])
            translate([0,-12,0])
                cube([28.5, 2,2.5], center = true);
            
            rotate([0,0,45])
            translate([0,-8,0])
                cube([33, 2,2.5], center = true);
            
            rotate([0,0,45])
            translate([0,-4,0])
                cube([36, 2,2.5], center = true);
            
            rotate([0,0,45])
                cube([36, 2,2.5], center = true);
            
            rotate([0,0,45])
            translate([0,4,0])
                cube([36, 2,2.5], center = true);
            
            rotate([0,0,45])
            translate([0,8,0])
                cube([33, 2,2.5], center = true);
            
            rotate([0,0,45])
            translate([0,12,0])
                cube([28.5, 2,2.5], center = true);
            
            rotate([0,0,45])
            translate([0,16,0])
                cube([19.5, 2,2.5], center = true);
        }
    }
    
    translate([50.5,0,0]) { // Keypad cavities
        translate([-9.525*3,9.525,0])
            cube([16.55,16.55,7], center=true);
        translate([-9.525*3,-9.525,0])
            cube([16.55,16.55,7], center=true);
        translate([-9.525*3,28.575,0])
            cube([16.55,16.55,7], center=true);
        translate([-9.525*3,-28.575,0])
            cube([16.55,16.55,7], center=true);
        
        translate([19.05,0,0]) {
            translate([-9.525*3,9.525,0])
                cube([16.55,16.55,7], center=true);
            translate([-9.525*3,-9.525,0])
                cube([16.55,16.55,7], center=true);
            translate([-9.525*3,28.575,0])
                cube([16.55,16.55,7], center=true);
            translate([-9.525*3,-28.575,0])
                cube([16.55,16.55,7], center=true);
        }
        
        translate([19.05*2,0,0]) {
            translate([-9.525*3,9.525,0])
                cube([16.55,16.55,7], center=true);
            translate([-9.525*3,-9.525,0])
                cube([16.55,16.55,7], center=true);
            translate([-9.525*3,28.575,0])
                cube([16.55,16.55,7], center=true);
            translate([-9.525*3,-28.575,0])
                cube([16.55,16.55,7], center=true);
        }
        
        translate([19.05*3,0,0]) {
            translate([-9.525*3,9.525,0])
                cube([16.55,16.55,7], center=true);
            translate([-9.525*3,-9.525,0])
                cube([16.55,16.55,7], center=true);
            translate([-9.525*3,28.575,0])
                cube([16.55,16.55,7], center=true);
            translate([-9.525*3,-28.575,0])
                cube([16.55,16.55,7], center=true);
        }
    }
    
    { // screw holes
        translate([95.5,70.5,0])
            cylinder(h=4.25, r = 2, center=true);
        translate([-95.5,70.5,0])
            cylinder(h=4.25, r = 2, center=true);
        translate([95.5,-70.5,0])
            cylinder(h=4.25, r = 2, center=true);
        translate([-95.5,-70.5,0])
            cylinder(h=4.25, r = 2, center=true);
    }
}


    translate([0,0,-.25]) { // pot text inset
        color([1.0,1.0,1.0,1])
        translate([-25,-54,.55])
        rotate([0,0,90])
        linear_extrude(.395,center=true)
        text("VOL/IO", size = 6, halign = "center", valign= "center");
        
        color([1.0,1.0,1.0,1.0])
        translate([25,-57,.55])
        rotate([0,0,90])
        linear_extrude(.395,center=true)
        text("KEY/SND", size = 6, halign = "center", valign= "center");
        
        { // mic text inset
        color([1.0,1.0,1.0,1.0])
        translate([2,-16,.55])
        rotate([0,0,90])
        linear_extrude(.399,center=true)
        text("MIC", size = 6, halign = "center", valign= "center");
        }
        
        { // tof text inset
            color([1.0,1.0,1.0,1.0])
            translate([-92,0,.55])
            rotate([0,0,90])
            linear_extrude(.395,center=true)
            text("TOF", size = 6, halign = "center", valign= "center");
        }
    
        { // IO text
            color([1.0,1.0,1.0,1.0])
            translate([-88,46,.55])
            rotate([0,0,90])
            linear_extrude(.395,center=true)
            text("EXT: IN | OUT", size = 6, halign = "center", valign= "center");
        }
        
        { // PWR text
            color([1.0,1.0,1.0,1.0])
            translate([-88,-54,.55])
            rotate([0,0,90])
            linear_extrude(.395,center=true)
            text("PWR", size = 6, halign = "center", valign= "center");
        }
        
    }
    
}

    