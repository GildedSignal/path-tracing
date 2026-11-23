#!/usr/bin/perl
# Code and test lines of the .rs files named on stdin: non-blank, non-comment; test lines are
# #[cfg(test)] items and files under tests/, benches/ or examples/. Prints "code tests".
use strict; use warnings;
my ($code, $test) = (0, 0);
for my $file (map { chomp; $_ } <STDIN>) {
    open my $fh, '<', $file or die "$file: $!";
    my $in_tests = $file =~ m{/tests/|/benches/|/examples/};
    my ($skip, $depth, $armed) = (0, 0, 0);
    while (my $line = <$fh>) {
        (my $bare = $line) =~ s{//.*}{}; $bare =~ s/"(?:\\.|[^"\\])*"//g; $bare =~ s/'(?:\\.|[^'\\])'//g;
        my $blank = $line =~ /^\s*$/ || $line =~ m{^\s*//};
        if (!$skip && $line =~ /^\s*#\[cfg\(test\)\]/) { $armed = 1; $test++; next; }
        if ($armed || $skip) {
            my $open = () = $bare =~ /\{/g; my $close = () = $bare =~ /\}/g;
            $test++ unless $blank;
            if ($armed) {
                if ($open) { $armed = 0; $skip = 1; $depth = $open - $close; $skip = 0 if $depth <= 0; }
                elsif ($bare =~ /;\s*$/) { $armed = 0; }
            } else { $depth += $open - $close; $skip = 0 if $depth <= 0; }
            next;
        }
        next if $blank;
        if ($in_tests) { $test++ } else { $code++ }
    }
}
print "$code $test\n";
