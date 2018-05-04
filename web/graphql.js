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

client.query({
  query: gql`
        query {
            status {
                bootTime,
                startupTime,
                scanDuration,
            }
        }
    `,
      variables: {}
    }).then(data => {
        console.log("DATA", data)
    })
// }).subscribe({
//   next (data) {
//     console.log("Status update", data)
//   }
// });
