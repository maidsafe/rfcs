#!/bin/bash

function generate {

  for file in $( grep -l -r --include="*.md" "\- Status: $1" text);
  do
    num=$(echo $file | sed -e "s/.*\/\([0-9]*\)-.*\.md/\1/")
    feature=$(head -n1 $file | sed -e "s/.*Feature Name: \(.*\)$/\1/")
    echo " - [#$num $feature](./$file)"
  done
}

echo "# RFCs by current status"
echo ""
echo "A list of all RFCs by their current status."
echo ""
echo "## Active RFCs"
echo ""
generate 'active'
echo ""
echo "## Proposed RFCs"
echo ""
generate 'proposed'
echo ""
echo "## Agreed RFCs"
echo ""
generate 'agreed'
echo ""
echo "## Implemented RFCs"
echo ""
generate 'implemented'
echo ""
echo "## Rejected RFCs"
echo ""
generate 'rejected'

echo ""
echo ""
echo "(Last updated _`date`_ at REV [`git rev-parse --short HEAD`](https://github.com/maidsafe/rfcs/commit/`git rev-parse HEAD`))"
