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
          <form class="navbar-form navbar-right { error ? 'bg-danger' : ''}">
            (<span class="glyphicon glyphicon-hdd" />
                { stats.machine.load_avg_1min.toFixed(2) } /
                { stats.machine.load_avg_5min.toFixed(2) } /
                { stats.machine.load_avg_15min.toFixed(2) }
                up { box_up() })
            (<span class="glyphicon glyphicon-scale" />
                up { up() })
            <request-error if={ error != null } error={ error } /></li>
            <a class="btn btn-default" href="/status">Status</a>
          </form>
        </div><!-- /.navbar-collapse -->
      </div><!-- /.container-fluid -->
    </nav>

    <script>
        import {RefreshJson} from "util/updater"
        import {from_ms, till_now_ms, format_uptime} from "util/time"

        status = "Loading"

        new RefreshJson("/status.json")
        .on("json_update", (value) => {
            this.error = null
            this.stats = value
            this.update()
        })
        .on("json_error", (err) => {
            this.error = err
            this.status = "ERR"
            this.update()
        })

        this.up = () => {
            return format_uptime(till_now_ms(from_ms(this.stats.startup_time)))
        }
        this.box_up = () => {
            return format_uptime(this.stats.machine.uptime*1000);
        }
    </script>

</cantal-nav>
