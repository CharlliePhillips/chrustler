$fn = 64;
include<BOSL2/std.scad>
translate([0,0,9.9])
difference() {
    cylinder(h=19.8, r = 2.75, center=true);
    
    translate([0,0,6.9])
    cylinder(h=6.1, r = 1.7, center=true);
    
    translate([0,0,-8])
    cylinder(h=4, r = 1.6, center=true);
}
//translate([5,5,4])
//union() {
//    cylinder(h=8, r = 2.75, center=true);
    
//    translate([0,0,6])
//    cylinder(h=4, r = 1.175, center=true);
//}