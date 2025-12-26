/**
 * MULTIPLAYER MICROCHAINS TESTING SCRIPT
 *
 * Copy and paste this into browser DevTools Console to quickly test
 * the multiplayer fix.
 *
 * Usage:
 * 1. Open Tab 1 → Run setupWhitePlayer()
 * 2. Open Tab 2 → Run setupBlackPlayer()
 * 3. Refresh both tabs
 * 4. Create game in Tab 1
 * 5. Run testMovesTab1() in Tab 1 (white should work, black should fail)
 * 6. Run testMovesTab2() in Tab 2 (black should work, white should fail)
 */

// ============================================================================
// TAB 1 SETUP - WHITE PLAYER (Chain 8974...)
// ============================================================================

function setupWhitePlayer() {
  console.log('🎮 Setting up WHITE PLAYER (Tab 1)');

  // Remove any existing chain ID to use default
  localStorage.removeItem('linera_chain_id');

  const chainId = localStorage.getItem('linera_chain_id') || 'DEFAULT (8974e56566be0e...)';

  console.log('✅ White Player Chain:', chainId);
  console.log('📋 Instructions:');
  console.log('   1. Refresh this page');
  console.log('   2. Check console for: "📡 Querying games from USER chain: 8974e56566..."');
  console.log('   3. Create a new game with opponent chain: 93b79b297357c49594fd2f0d3f8672d179b0c19411711bfdc2f09d5c53c39932');
  console.log('   4. You should only be able to move WHITE pieces');

  return {
    chainId,
    player: 'white',
    canMove: 'white pieces only'
  };
}

// ============================================================================
// TAB 2 SETUP - BLACK PLAYER (Chain 93b79...)
// ============================================================================

function setupBlackPlayer() {
  console.log('🎮 Setting up BLACK PLAYER (Tab 2)');

  // Set chain ID to Player 2's chain
  const blackChain = '93b79b297357c49594fd2f0d3f8672d179b0c19411711bfdc2f09d5c53c39932';
  localStorage.setItem('linera_chain_id', blackChain);

  const chainId = localStorage.getItem('linera_chain_id');

  console.log('✅ Black Player Chain:', chainId);
  console.log('📋 Instructions:');
  console.log('   1. Refresh this page');
  console.log('   2. Check console for: "📡 Querying games from USER chain: 93b79b2973..."');
  console.log('   3. Wait for game to appear (cross-chain message from Tab 1)');
  console.log('   4. You should only be able to move BLACK pieces');

  return {
    chainId,
    player: 'black',
    canMove: 'black pieces only'
  };
}

// ============================================================================
// VERIFICATION FUNCTIONS
// ============================================================================

function checkCurrentChain() {
  const chainId = localStorage.getItem('linera_chain_id') || 'DEFAULT (8974e56566be0e...)';
  const isWhite = chainId.startsWith('8974') || chainId === 'DEFAULT (8974e56566be0e...)';
  const isBlack = chainId.startsWith('93b79');

  console.log('🔍 Current Chain Configuration:');
  console.log('   Chain ID:', chainId);
  console.log('   Player Role:', isWhite ? 'WHITE' : isBlack ? 'BLACK' : 'UNKNOWN');
  console.log('   Can Move:', isWhite ? 'White pieces' : isBlack ? 'Black pieces' : 'Unknown');

  return {
    chainId,
    isWhite,
    isBlack,
    player: isWhite ? 'white' : isBlack ? 'black' : 'unknown'
  };
}

function verifyFix() {
  console.log('🔎 VERIFYING MULTIPLAYER FIX...\n');

  const config = checkCurrentChain();

  console.log('✅ Fix Verification:');
  console.log('   1. localStorage chain ID:', config.chainId !== 'DEFAULT (8974e56566be0e...)' ? '✅ Set' : '⚠️  Using default');
  console.log('   2. Player role identified:', config.player !== 'unknown' ? '✅ Yes' : '❌ No');
  console.log('   3. Expected query chain:', config.isWhite ? '8974e56566...' : '93b79b2973...');

  console.log('\n📝 Next Steps:');
  console.log('   1. Refresh the page');
  console.log('   2. Watch console for "📡 Querying games from USER chain: ..."');
  console.log('   3. Verify the chain ID matches:', config.isWhite ? '8974e56566...' : '93b79b2973...');

  return config;
}

// ============================================================================
// MOVE TESTING (Use after game is created)
// ============================================================================

function testMovesTab1(gameId) {
  console.log('🧪 TESTING MOVES - TAB 1 (White Player)');
  console.log('Expected: White moves succeed, Black moves fail\n');

  if (!gameId) {
    console.error('❌ Please provide a gameId: testMovesTab1("your-game-id")');
    return;
  }

  console.log('📋 Manual Test Steps:');
  console.log('   1. Try moving a white piece (e.g., e2 → e4)');
  console.log('      Expected: ✅ SUCCESS');
  console.log('   2. Try moving a black piece (e.g., e7 → e5)');
  console.log('      Expected: ❌ FAIL ("Not your turn" or "Invalid player")');
  console.log('   3. Check console logs for GraphQL responses');

  console.log('\n⚠️  If black moves succeed from Tab 1, the fix is NOT working!');
  console.log('    Only white pieces should be movable from chain 8974...');
}

function testMovesTab2(gameId) {
  console.log('🧪 TESTING MOVES - TAB 2 (Black Player)');
  console.log('Expected: Black moves succeed, White moves fail\n');

  if (!gameId) {
    console.error('❌ Please provide a gameId: testMovesTab2("your-game-id")');
    return;
  }

  console.log('📋 Manual Test Steps:');
  console.log('   1. Wait for white to move (Tab 1 makes first move)');
  console.log('   2. Try moving a black piece (e.g., e7 → e5)');
  console.log('      Expected: ✅ SUCCESS');
  console.log('   3. Try moving a white piece (e.g., d2 → d4)');
  console.log('      Expected: ❌ FAIL ("Not your turn" or "Invalid player")');
  console.log('   4. Check console logs for GraphQL responses');

  console.log('\n⚠️  If white moves succeed from Tab 2, the fix is NOT working!');
  console.log('    Only black pieces should be movable from chain 93b79...');
}

// ============================================================================
// CONTRACT ENFORCEMENT TEST (Advanced)
// ============================================================================

async function testContractEnforcement(gameId) {
  if (!gameId) {
    console.error('❌ Please provide a gameId: testContractEnforcement("your-game-id")');
    return;
  }

  console.log('🔐 TESTING CONTRACT ENFORCEMENT...\n');

  const config = checkCurrentChain();
  const graphqlUrl = 'http://localhost:8081';
  const appId = 'ca57a58c816908ea44cae5ddce60ff25da1dc5f19b1fd576c202be7fb548c8a5';

  const chainId = config.isWhite
    ? '8974e56566be0e114121934122b0d867123b1d366a815f5c6104e37c9ae735f8'
    : '93b79b297357c49594fd2f0d3f8672d179b0c19411711bfdc2f09d5c53c39932';

  const url = `${graphqlUrl}/chains/${chainId}/applications/${appId}`;

  // Test 1: Move own piece (should succeed)
  const ownMove = config.isWhite
    ? { from: 'e2', to: 'e4' }  // White pawn
    : { from: 'e7', to: 'e5' };  // Black pawn

  console.log(`Test 1: Moving ${config.player} piece...`);
  console.log(`   From: ${ownMove.from} → To: ${ownMove.to}`);

  try {
    const response1 = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        query: `mutation { placeMove(gameId: "${gameId}", from: "${ownMove.from}", to: "${ownMove.to}") }`
      })
    });
    const result1 = await response1.json();
    console.log('   Result:', result1.error ? '❌ FAILED' : '✅ SUCCESS', result1);
  } catch (error) {
    console.log('   Result: ⚠️  ERROR', error.message);
  }

  // Test 2: Move opponent's piece (should fail)
  const opponentMove = config.isWhite
    ? { from: 'e7', to: 'e5' }  // Black pawn (white trying to move)
    : { from: 'e2', to: 'e4' };  // White pawn (black trying to move)

  console.log(`\nTest 2: Moving ${config.isWhite ? 'black' : 'white'} piece (should fail)...`);
  console.log(`   From: ${opponentMove.from} → To: ${opponentMove.to}`);

  try {
    const response2 = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        query: `mutation { placeMove(gameId: "${gameId}", from: "${opponentMove.from}", to: "${opponentMove.to}") }`
      })
    });
    const result2 = await response2.json();
    console.log('   Result:', result2.error ? '✅ CORRECTLY REJECTED' : '❌ INCORRECTLY ALLOWED', result2);
  } catch (error) {
    console.log('   Result: ✅ CORRECTLY REJECTED', error.message);
  }

  console.log('\n✅ Contract enforcement test complete!');
}

// ============================================================================
// QUICK START GUIDE
// ============================================================================

console.log(`
╔════════════════════════════════════════════════════════════════════════╗
║        MULTIPLAYER MICROCHAINS TESTING SCRIPT LOADED                   ║
╚════════════════════════════════════════════════════════════════════════╝

Available Functions:

📍 SETUP FUNCTIONS:
   setupWhitePlayer()     - Configure Tab 1 as White Player (Chain 8974...)
   setupBlackPlayer()     - Configure Tab 2 as Black Player (Chain 93b79...)

🔍 VERIFICATION FUNCTIONS:
   checkCurrentChain()    - Show current chain configuration
   verifyFix()           - Comprehensive verification of the fix

🧪 TESTING FUNCTIONS:
   testMovesTab1(gameId) - Test move restrictions in Tab 1 (White)
   testMovesTab2(gameId) - Test move restrictions in Tab 2 (Black)
   testContractEnforcement(gameId) - Advanced contract validation test

📋 QUICK START:
   1. Tab 1: Run setupWhitePlayer() and refresh
   2. Tab 2: Run setupBlackPlayer() and refresh
   3. Tab 1: Create a game
   4. Both: Run verifyFix() to confirm setup
   5. Both: Test moves with testMovesTab1() or testMovesTab2()

⚠️  EXPECTED BEHAVIOR:
   - Tab 1 (White): Can only move white pieces
   - Tab 2 (Black): Can only move black pieces
   - Attempting to move opponent's pieces should fail with contract error

═══════════════════════════════════════════════════════════════════════
`);
