export const startJob = /* GraphQL */ `
  mutation StartJob($id: String!) {
    startJob(id: $id) {
      id
      status
      message
      createdAt
      updatedAt
      __typename
    }
  }
`;

export const subscribeStartJob = /* GraphQL */ `
  subscription SubscribeStartJob($id: String!) {
    subscribeStartJob(id: $id) {
      id
      status
      message
      createdAt
      updatedAt
      __typename
    }
  }
`;

export const subscribeCompleteJob = /* GraphQL */ `
  subscription SubscribeCompleteJob($id: String!) {
    subscribeCompleteJob(id: $id) {
      id
      status
      message
      createdAt
      updatedAt
      __typename
    }
  }
`;
