import { ApolloClient } from 'apollo-client';
import { InMemoryCache } from 'apollo-cache-inmemory';
import { WebSocketLink } from "apollo-link-ws";
import { SubscriptionClient } from "subscriptions-transport-ws";
import gql from 'graphql-tag';

const GRAPHQL_ENDPOINT = "ws://"+location.host+"/graphql";

const ws_client = new SubscriptionClient(GRAPHQL_ENDPOINT, {
  reconnect: true
});

const link = new WebSocketLink(ws_client);

const client = new ApolloClient({
    link,
    cache: new InMemoryCache(),
});

export var beacon

let q = client.subscribe({
  query: gql`
        subscription {
            beacon: status {
                id,
                version,
                hostname,
                name,
                clusterName,
                bootTime,
                startupTime,
                scanDuration,
                currentTime,
                selfReport,
                threadsReport,
                tipValues,
                fineValues,
                processes,
            }
        }
    `,
  variables: {}
}).subscribe({
  next (data) {
    let time = Date.now()
    beacon = data.data.beacon
    beacon.latency = beacon.currentTime - time
  }
});
