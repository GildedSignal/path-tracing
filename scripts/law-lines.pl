#!/usr/bin/perl
# Code lines of the law: non-blank, non-comment, outside #[cfg(test)] items.
# perl scripts/law-lines.pl [files...]  (default: every .rs under engine/law/src), prints code / total.
use strict; use warnings; use File::Find;
my @files = @ARGV;
find(sub { push @files, $File::Find::name if /\.rs$/ }, 'engine/law/src') unless @files;
my ($code, $total) = (0, 0);
for my $file (sort @files) {
    open my $fh, '<', $file or die "$file: $!";
    my ($skip, $depth, $armed, $c) = (0, 0, 0, 0);
    while (my $line = <$fh>) {
        $total++;
        (my $bare = $line) =~ s{//.*}{}; $bare =~ s/"(?:\\.|[^"\\])*"//g; $bare =~ s/'(?:\\.|[^'\\])'//g;
        if (!$skip && $line =~ /^\s*#\[cfg\(test\)\]/) { $armed = 1; next; }
        if ($armed) {
            my $open = () = $bare =~ /\{/g; my $close = () = $bare =~ /\}/g;
            if ($open) { $armed = 0; $skip = 1; $depth = $open - $close; $skip = 0 if $depth <= 0; next; }
            $armed = 0 if $bare =~ /;\s*$/; next;
        }
        if ($skip) {
            my $open = () = $bare =~ /\{/g; my $close = () = $bare =~ /\}/g;
            $depth += $open - $close; $skip = 0 if $depth <= 0; next;
        }
        next if $line =~ /^\s*$/ || $line =~ m{^\s*//};
        $c++;
    }
    $code += $c;
    print "$c\t$file\n" if $ENV{PER_FILE};
}
print "$code / $total\n";
