<cantal-nav>
    <nav class="navbar navbar-default">
      <div class="container-fluid">
        <!-- Brand and toggle get grouped for better mobile display -->
        <div class="navbar-header">
          <a class="navbar-brand" href="#">Cantal</a>
        </div>

        <!-- Collect the nav links, forms, and other content for toggling -->
        <div class="collapse navbar-collapse" id="bs-example-navbar-collapse-1">
          <ul class="nav navbar-nav">
            <li class="active"><a href="#">Active
              <span class="sr-only">(current)</span></a></li>
            <li><a href="#">Inactive</a></li>
          </ul>
          <ul class="nav navbar-nav navbar-right { error ? 'bg-danger' : ''}">
            <li><a href="#">{ status }
                <request-error if={ error != null } error={ error } /></a></li>
          </ul>
        </div><!-- /.navbar-collapse -->
      </div><!-- /.container-fluid -->
    </nav>

    <script>
        import {RefreshJson} from "util/updater"

        status = "Loading"

        new RefreshJson("/status.json")
        .on("json_update", (value) => {
            this.error = null
            this.status = "value" + value.startup_time;
            this.update()
        })
        .on("json_error", (err) => {
            this.error = err
            this.status = "ERR"
            this.update()
        })
    </script>

</cantal-nav>
