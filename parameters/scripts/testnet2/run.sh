# These are dependent on each other.
./universal.sh || exit
./noop.sh || exit
./inner.sh || exit
./outer.sh || exit

./posw.sh || exit

./genesis.sh || exit
