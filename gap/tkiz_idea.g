LoadPackage("numericalsgps");
LoadPackage("intpic");

ns := NumericalSemigroup(10, 15, 19, 81);;
highlights := ["conductor", "min_generators", "small_elements"];;
options := rec(ns_table := true, colors := ["blue", "red!70", "-red", "black!40"]);;

tkz := TikzCodeForNumericalSemigroup(ns, highlights, options);;

# Preview it (opens a PDF viewer)
IP_Splash(tkz);

# Or save the tikz code to a file
FileString("my_semigroup.tex", tkz);
