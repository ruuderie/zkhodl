<script>
    import { onMount } from 'svelte';
    import { bitcoin } from '@scure/btc-signer';
    
    let address = '';
    let thresholdAmount = 0;
    let proofStatus = '';
    let loading = false;
  
    async function connectWallet() {
      try {
        // Request wallet connection (assuming window.bitcoin exists)
        const accounts = await window.bitcoin.request({ 
          method: 'requestAccounts' 
        });
        address = accounts[0];
      } catch (error) {
        console.error('Failed to connect wallet:', error);
      }
    }
  
    async function generateProof() {
      loading = true;
      proofStatus = 'Generating proof...';
      
      try {
        // Get signature from connected wallet
        const signature = await window.bitcoin.request({
          method: 'signMessage',
          params: [address, 'Verify balance ownership']
        });
  
        // Generate proof
        const response = await fetch('http://localhost:8080/generate-proof', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json'
          },
          body: JSON.stringify({
            address,
            signature,
            threshold_amount: thresholdAmount
          })
        });
  
        if (!response.ok) throw new Error('Failed to generate proof');
        
        const { proof } = await response.json();
        proofStatus = 'Proof generated successfully!';
        
        // Store proof or proceed with verification
        await verifyProof(proof);
      } catch (error) {
        proofStatus = `Error: ${error.message}`;
      } finally {
        loading = false;
      }
    }
  
    async function verifyProof(proof) {
      proofStatus = 'Verifying proof...';
      
      try {
        const response = await fetch('http://localhost:8080/verify-proof', {
          method: 'POST',
          body: proof
        });
  
        if (!response.ok) throw new Error('Proof verification failed');
        
        proofStatus = 'Proof verified successfully!';
      } catch (error) {
        proofStatus = `Verification error: ${error.message}`;
      }
    }
  </script>
  
  <main class="container mx-auto p-4">
    <h1 class="text-2xl font-bold mb-4">ZK Bitcoin Balance Verifier</h1>
  
    <div class="mb-4">
      {#if !address}
        <button 
          on:click={connectWallet}
          class="bg-blue-500 text-white px-4 py-2 rounded"
        >
          Connect Bitcoin Wallet
        </button>
      {:else}
        <p>Connected Address: {address}</p>
      {/if}
    </div>
  
    <div class="mb-4">
      <label class="block mb-2">Threshold Amount (BTC)</label>
      <input 
        type="number"
        bind:value={thresholdAmount}
        class="border p-2 rounded w-full"
        step="0.00000001"
        min="0"
      />
    </div>
  
    <button 
      on:click={generateProof}
      disabled={!address || loading}
      class="bg-green-500 text-white px-4 py-2 rounded disabled:opacity-50"
    >
      {loading ? 'Processing...' : 'Generate Proof'}
    </button>
  
    {#if proofStatus}
      <div class="mt-4 p-4 border rounded">
        <p>{proofStatus}</p>
      </div>
    {/if}
  </main>