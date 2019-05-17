# csv_to_gv_aon
Create an Activity-on-Node diagram from a CSV file

## How to use
This program does not actually draw the diagram itself, it actually outputs <a href="https://en.wikipedia.org/wiki/DOT_(graph_description_language)">DOT</a>, which can be piped into the `dot` program from GraphViz. One way to actually generate an image is: `./csv_to_gv_aon /path/to/csv | dot -Tpng -o out.png`. This program supports one command line option `--dslack`, which will put the slack value on both sides of the description.

## Sample input

| Activity | Description | Duration | Predecessor |
| --- | --- | --- | --- |
| 1 | 1st activity | 2 | |
| 2 | Burst activity | 4 | 1 |
| 3 | 3rd activity | 3 | 2 |
| 4 | 4th activity | 1 | 2 |
| 5 | Merge activity | 5 | 3,4 |

## Sample output
![sample output](sample_output.png)

### With the `--dslack` option:
![sample output with --dslack](sample_output_dslack.png)

## Output format
![output format](output_format.png)

### With the `--dslack` option:
![output format with --dslack](output_format_dslack.png)