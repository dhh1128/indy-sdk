package org.hyperledger.indy.sdk;

import org.hyperledger.indy.sdk.agent.Agent;
import org.hyperledger.indy.sdk.agent.Agent.Connection;
import org.hyperledger.indy.sdk.agent.Agent.Listener;
import org.hyperledger.indy.sdk.agent.AgentObservers.ConnectionObserver;
import org.hyperledger.indy.sdk.agent.AgentObservers.ListenerObserver;
import org.hyperledger.indy.sdk.agent.AgentObservers.MessageObserver;
import org.hyperledger.indy.sdk.ledger.Ledger;
import org.hyperledger.indy.sdk.pool.Pool;
import org.hyperledger.indy.sdk.signus.Signus;
import org.hyperledger.indy.sdk.signus.SignusJSONParameters;
import org.hyperledger.indy.sdk.signus.SignusResults;
import org.hyperledger.indy.sdk.utils.PoolUtils;
import org.hyperledger.indy.sdk.wallet.Wallet;
import org.junit.Test;

import static org.junit.Assert.assertNotNull;

public class AgentTest extends IndyIntegrationTest {

	@Test
	public void testAgentDemo() throws Exception {
		String endpoint = "127.0.0.1:9801";

		Pool pool = PoolUtils.createAndOpenPoolLedger();
		assertNotNull(pool);
		openedPools.add(pool);

		Wallet.createWallet("test" /* FIXME */, "trustee_wallet", null, null, null).get();
		Wallet.createWallet("test" /* FIXME */, "listener_wallet", null, null, null).get();
		Wallet trusteeWallet = Wallet.openWallet("trustee_wallet", null, null).get();
		assertNotNull(trusteeWallet);
		Wallet listenerWallet = Wallet.openWallet("listener_wallet", null, null).get();
		assertNotNull(listenerWallet);
		Wallet senderWallet = trusteeWallet;

		SignusJSONParameters.CreateAndStoreMyDidJSONParameter myDidJSONParameter
				= new SignusJSONParameters.CreateAndStoreMyDidJSONParameter(null, IndyIntegrationTest.TRUSTEE_SEED, null, false);
		SignusResults.CreateAndStoreMyDidResult trustee = Signus.createAndStoreMyDid(trusteeWallet, myDidJSONParameter.toJson()).get();

		SignusResults.CreateAndStoreMyDidResult listener = Signus.createAndStoreMyDid(listenerWallet, "{}").get();

		SignusResults.CreateAndStoreMyDidResult sender = trustee;

		String listenerNymJson = Ledger.buildNymRequest(trustee.getDid(), listener.getDid(), listener.getVerkey(), null, null).get();

		Ledger.signAndSubmitRequest(pool, trusteeWallet, trustee.getDid(), listenerNymJson).get();

		String listenerAttribJson = Ledger.buildAttribRequest(listener.getDid(), listener.getDid(), null,
				String.format("{\"endpoint\":{\"ha\":\"%s\",\"verkey\":\"%s\"}}", endpoint, listener.getPk()), null).get();

		Ledger.signAndSubmitRequest(pool, listenerWallet, listener.getDid(), listenerAttribJson).get();

		final MessageObserver messageObserver = new MessageObserver() {

			public void onMessage(Connection connection, String message) {

				System.out.println("Received message '" + message + "' on connection " + connection);
			}
		};

		final ConnectionObserver connectionObserver = new ConnectionObserver() {

			public MessageObserver onConnection(Listener listener, Connection connection, String senderDid, String receiverDid) {

				System.out.println("New connection " + connection);

				return messageObserver;
			}
		};

		final ListenerObserver listenerObserver = new ListenerObserver() {

			public ConnectionObserver onListener(Listener aListener) {

				System.out.println("New listener " + aListener);
				try {
					Agent.agentAddIdentity(aListener, pool, listenerWallet, listener.getDid());
				} catch (IndyException e) {
					e.printStackTrace();
				}

				return connectionObserver;
			}
		};

		Agent.agentListen(endpoint, listenerObserver);

		Agent.agentConnect(pool, senderWallet, sender.getDid(), listener.getDid(), connectionObserver);

		Thread.sleep(500); /* FIXME */
	}
}
