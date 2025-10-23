# PyDigger implemented in Rust

* Get the details in the json file
    * For https://pypi.org/project/txt2detection/1.0.6/
    * use this address https://pypi.org/pypi/txt2detection/1.0.6/json


* Create reports in JSON format and build a front-end that can show the data requiring only front-end code.
* From all the project JSON files
    * Total number of projects
    * Number of projects that have no `home_page` field
    * Check if the `home_page` field is one of the well-known VCS systems.
    * Download the source code from pypi
    * Run `mypy` on the code and report the errors
    * List the packages that have VCS but have problems with `mypy`

