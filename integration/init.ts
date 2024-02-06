import axios from 'axios'

interface TestSetup {
  baseUrl: string;
  projectId: string;
  httpClient: any;
}

export const getTestSetup = (): TestSetup => {
  const baseUrl = process.env.RPC_URL;
  if (!baseUrl) {
    throw new Error('RPC_URL environment variable not set');
  }
  const projectId = process.env.PROJECT_ID;
  if (!projectId) {
    throw new Error('PROJECT_ID environment variable not set');
  }
  const httpClient = axios.create({
    validateStatus: (_status) => true,
  })

  return { baseUrl, projectId, httpClient };
};
