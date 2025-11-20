$fn=32;

difference() {
    import("chrustler_keycap_base.stl");

    translate([0,0,7])
    linear_extrude(.4)
        text("I", size=6, halign = "center", valign = "center");
}

translate([20,0,0])
difference() {
    import("chrustler_keycap_base.stl");

    translate([0,0,7])
    linear_extrude(.4)
        text("II", size=6, halign = "center", valign = "center");
}

translate([40,0,0])
difference() {
    import("chrustler_keycap_base.stl");

    translate([0,0,7])
    linear_extrude(.4)
        text("III", size=6, halign = "center", valign = "center");
}

translate([60,0,0])
difference() {
    import("chrustler_keycap_base.stl");

    translate([0,0,7])
    linear_extrude(.4)
        text("mod", size=5, halign = "center", valign = "center");
}

translate([0,-20,0])
difference() {
    import("chrustler_keycap_base.stl");

    translate([0,0,7])
    linear_extrude(.4)
        text("IV", size=6, halign = "center", valign = "center");
}

translate([20,-20,0])
difference() {
    import("chrustler_keycap_base.stl");

    translate([0,0,7])
    linear_extrude(.4)
        text("V", size=6, halign = "center", valign = "center");
}

translate([40,-20,0])
difference() {
    import("chrustler_keycap_base.stl");

    translate([0,0,7])
    linear_extrude(.4)
        text("VI", size=6, halign = "center", valign = "center");
}

translate([60,-20,0])
difference() {
    import("chrustler_keycap_base.stl");

    translate([0,0,7])
    linear_extrude(.4)
        text("gat", size=6, halign = "center", valign = "center");
}

translate([0,-40,0])
difference() {
    import("chrustler_keycap_base.stl");

    translate([0,0,7])
    linear_extrude(.4)
        text("VII", size=6, halign = "center", valign = "center");
}

translate([20,-40,0])
difference() {
    import("chrustler_keycap_base.stl");

    translate([0,0,7])
    linear_extrude(.4)
        text("<oc", size=5, halign = "center", valign = "center");
}

translate([40,-40,0])
difference() {
    import("chrustler_keycap_base.stl");

    translate([0,0,7])
    linear_extrude(.4)
        text("oc>", size=5, halign = "center", valign = "center");
}

translate([60,-40,0])
difference() {
    import("chrustler_keycap_base.stl");

    translate([0,0,7])
    linear_extrude(.4)
        text("tof", size=6, halign = "center", valign = "center");
}

translate([0,-60,0])
difference() {
    import("chrustler_keycap_base.stl");

    translate([0,0,7.175])
    //linear_extrude(.4)
    //    text("O", size=6, halign = "center", valign = "center");
    cylinder(h=.4,r=3,center=true);
}

translate([20,-60,0])
difference() {
    import("chrustler_keycap_base.stl");

    translate([0,0,7])
    linear_extrude(.4)
        text("f", size=6, halign = "center", valign = "center");
}

translate([40,-60,0])
difference() {
    import("chrustler_keycap_base.stl");

    translate([0,0,7])
    linear_extrude(.4)
        text("hld", size=6, halign = "center", valign = "center");
}

translate([60,-60,0])
difference() {
    import("chrustler_keycap_base.stl");

    translate([0,0,7])
    linear_extrude(.4)
        text("typ", size=6, halign = "center", valign = "center");
}