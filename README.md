# PyDigger implemented in Rust

See [DEVELOPMENT](DEVELOPMENT.md)

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

* The report folder will contain a file called `report.json` that contains,
    * `total`
    * `stats`
* There will be a file called `projects.json` that lists all the project names.
* For each project there will be a file called `projects/<PROJECT>.json` with the collected meta-data
    * `version`
    * `vcs`

* The reports folder will be the `docs` folder of a separate repository
* There will also be a repository with the front-end code which is an HTML page and some JavaScript that will load the `report.json` on when loaded.


------

* Download RSS feed of recent uploads, in memory
* Download meta-data from each recently uploaded project, but only if we have not processed it yet. (We can chect this using the file we save in the next step)
* Extract some data from the meta-data that we would like to display and save it in a hashed folder that will be in a github repo.
* Download the package from pypi, analize it and save some more data in the per-project file.
* Download the git repository of the project and run some further analuzis on it and save the result in the per-project file.
* Collect some stats from the saved files and save that too in the "data-repo"


* Copy the files from the data repo to the web site
* Copy the HTML file and other static files to the web site.
* Copy the front-end code to the web site


## TODO

Collect
* Save how many were in the RSS feed in the most recent run.
* Save the start time of data collection.
* Save elapsed time of data collection.
* ----
* Save how many projects were download in the most recent run
* In the current run cache the names of the projects we see and only download each one once - this will be more important when we move to async.
* Save how many unique names were in the most recent RSS feed
* Switch data collection to be async

Report
* Show the total number of packages.
* List the N most recent packages.
* Create stats page with list of licenses, show a pre-defined list of licenses. List of projects with other licenses, list of projects without license.
* ----
* For each package create its own page
