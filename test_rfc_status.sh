#!/bin/bash

./generate_by_status.sh > generated
diff generated RFCs-by-status.md | wc -l > new_lines
count=`cat new_lines`

rm generated
rm new_lines
echo "number of changes to the RFCs-by-status.md = $count"

# if the differences aren't only the timestamp (4 lines) then exit code 1
if [ $count -ne 4 ]
then
  #fail
  echo "This has failed because it appears that there has been a change to a"
  echo "RFC document status, which has not been reflected in your PR."
  echo "Please run the generate_by_status.sh script and commit the updates"
  echo "as part of your PR"
  exit 1
fi
