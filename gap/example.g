
-- see https://gap-packages.github.io/numericalsgps/

LoadPackage("NumericalSgps");

ng := NumericalSemigroup(---here the a set of numbers -- input --));

m:= Multiplicity(ng);

f := FrobeniusNumber(ng);

e := EmbeddingDimension(ng);

num_gaps := GenusOfNumericalSemigroup(ng);

num_set := 1+f - num_gaps;

