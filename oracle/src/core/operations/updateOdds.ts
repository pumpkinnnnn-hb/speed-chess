import { GraphQLClient, gql } from 'graphql-request';
import { GameOdds } from '../types';
import { config } from '../../config';

export class OddsUpdater {
  private client: GraphQLClient;

  constructor() {
    this.client = new GraphQLClient(config.serviceUrl + '/chains/' + config.chainId + '/applications/' + config.bettingAppId);
  }

  async updateOdds(gameId: string, odds: GameOdds): Promise<boolean> {
    try {
      const mutation = gql`
        mutation UpdateOdds($gameId: String!, $evaluation: Int!) {
          updateOdds(gameId: $gameId, evaluation: $evaluation)
        }
      `;

      const variables = {
        gameId,
        evaluation: odds.evaluation,
      };

      const response = await this.client.request(mutation, variables);
      
      console.log('Odds updated for game:', gameId);
      console.log('  Evaluation:', odds.evaluation, 'cp');
      
      return true;
    } catch (error) {
      console.error('Failed to update odds for game:', gameId, error);
      return false;
    }
  }
}
