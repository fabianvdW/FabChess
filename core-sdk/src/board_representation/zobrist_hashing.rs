use crate::board_representation::game_state::{
    PieceType, CASTLE_BLACK_KS, CASTLE_BLACK_QS, CASTLE_WHITE_KS, CASTLE_WHITE_QS,
};
use rand::prelude::*;
use std::fmt::{Display, Formatter, Result};
use std::u64;

pub fn rand_u64(rng: &mut StdRng) -> u64 {
    let res: u64 = rng.gen();
    res
}
#[rustfmt::skip]
pub const ZOBRIST_KEYS : Zobrist = Zobrist{
    side_to_move: 0x3b38550242f1f674u64,
    castle_permissions: [0, 9281808104549773017, 4942683839431447707, 14148176465394924098, 3201500842666018998, 12439398771358377583, 7564346216580439085, 16732684083307426548, 17553584693864495093, 8310626812661143852, 13190247833429009262, 4017975914097872311, 16137761183284215619, 6862152125292073370, 11196837666809064408, 1994166387682690305],
    en_passant: [4621838342794099355, 13245742013170795564, 14076864459530701426, 11826998901654169481, 4940071235972956129, 2843563498410573842, 7493695263351978215, 10399002629402436129],
    pieces: [[[0x7fad605f8e82528cu64,0xd9d3050cf849da49u64,0xba93b15dfed7c67du64,0x9ae49c90e46beabdu64,0x6a7438d955b94a93u64,0x378f6149b61424e0u64,0x4a86bc99412e8584u64,0x35a821804b0c92a8u64,0xcf5c7e0382cf80u64,0x838b84e642bee88au64,0xdf8418b5515f6845u64,0x8f38716327150356u64,0x8d8f1a90d3a20f98u64,0x505283141adc432bu64,0x4f1362d7f19bb21du64,0x468102d133530919u64,0x179abb51fc995e73u64,0xe9f9e6f77770de6eu64,0xa7ed08f25fd19f4cu64,0x702f251d02badfa7u64,0x3561fe3f010f2d64u64,0x8c70a4a22de94e9fu64,0x10f5b41783d8e0a4u64,0xdb9d2a8937e61534u64,0xb073adbe4edd86bu64,0x1628dda4c64525edu64,0x2954154adfd98b5bu64,0xafb35ee8fe1b0429u64,0x5d7cd7ba61bf16cau64,0xa475b64242762590u64,0xb285f94b9753389au64,0x486d42bb68ef464eu64,0xd5c78efedf608ed4u64,0x5884773ead1ec8fdu64,0xd538c60f03bff7b0u64,0xd08156b2d28bd850u64,0x83e85618bd52471u64,0x7fbf800099f73664u64,0xaeb13a410050bfbdu64,0xb3a7f24be18670d9u64,0x8c91d7d02ab8d7eeu64,0xa71a099ceb2774feu64,0x8315f979dd4a1c79u64,0xccf772953209be86u64,0xca965558ce75e361u64,0x1905f1e96909e749u64,0xb536fdb8b3d31u64,0x33a78d19af01fe32u64,0x62d696a90b1113b7u64,0x49d468e7ed916dcbu64,0x72b90167037cd67du64,0x3537a442a2daaa12u64,0x8e4dc13930dee6cbu64,0xa3df5a2258ce6ce6u64,0xc508ba5e5a117cd3u64,0x5c191c14cfd28240u64,0xcc6b0a3e65a26758u64,0x2db5c9d831b59badu64,0xec96558c314e8f0u64,0xf7671ae63a183005u64,0xd3e4ac4dbe092b3eu64,0xcede54c6d750a599u64,0xba1c47a5e1fb24edu64,0x76e9ef8f23d0ed33u64,],[0x160283e5461f1998u64,0x7a69038897365444u64,0xd451891e1bb63a5eu64,0x5f4a61ab7be69afeu64,0x8d07b14475fdfc7bu64,0x6d2edf45ef7c39dau64,0xbc1e0805687498e4u64,0x29dba4ed00ab98fu64,0x1e872fd4570ef036u64,0xc6560fd932e88739u64,0x3df8e6837590b94u64,0xbe02c4ae0839fdc4u64,0x8db701a6062b94a3u64,0x2deb49e37276e394u64,0xe0196871d04fdda2u64,0x8b6dc1b29061c67fu64,0x69759e60d724c95bu64,0x2cd3a6b94b232ce4u64,0x5f0d756cf99d679eu64,0xb2510aa4b5258561u64,0xae3cfe00674c7315u64,0x4695eafb2fb7920du64,0xbfaf497e5fd32be7u64,0x7b1e5999c8673402u64,0xc0058b278003869eu64,0x64ef0a03caa0c2e6u64,0x8e2d19f10d147c2du64,0x243883deb95ac4efu64,0x954ed2cdb79890b8u64,0x8bc29eb36253cff3u64,0xe5c894ca195d8b2cu64,0xd7340f4306f8f950u64,0x89a258e57b3d13deu64,0xb05062d926d27ac4u64,0xacfa4ff93e9703a9u64,0x82221abbd0258e75u64,0x40eca7eb2be29fbdu64,0x9d790eab96c5d0bu64,0xa50809c7ffa51482u64,0xe9c1a123f4386e6cu64,0x796929bff5f2b022u64,0x6c46ec1bdbeada46u64,0x8d6d298586ba5ae2u64,0x3b975421b857b947u64,0x4ff803a00461b209u64,0x2722944add7d0852u64,0xd905a958cb3f1777u64,0x2ab934c3acc8f785u64,0xd2b03f6dd727ae9du64,0x2362f4df8830041bu64,0x6d59da3ad5136e1fu64,0xa54576852ba6dbdcu64,0xf2bc6df97b1711fu64,0x9c052d18d9444f12u64,0x66d81740c3133015u64,0x3d9a85979a08c740u64,0xe6557afa05bac328u64,0x6c4d5e3d77e0336au64,0xe49d7c1d2b9f1839u64,0xa5d48d5818b6d20bu64,0xc7211b9eb2ba8919u64,0x5392f1ba5963198cu64,0xcd865722b4b3678eu64,0x1e0b27ac9197f556u64,],[0x96784fd94f26cdf9u64,0x83f252e170391e8du64,0x589f452539958106u64,0xc9344f1400c1c042u64,0xf3f340a01f1b7b05u64,0xa374e8f33623276cu64,0x25b9480a20ef3e0fu64,0x978fcc99d9f6d093u64,0x6544deb79060aacdu64,0xbdcafd507283cff3u64,0x8698f3fbbd090057u64,0xb25ae56f61879cf4u64,0x8956481b158d38c2u64,0xe8d0a9ff933519a7u64,0x2a6705ecf5ad1aabu64,0x23dd92e10103461du64,0xd8310a1839a3e9f5u64,0xa848d33b910b18d6u64,0x14be5d6e635a7896u64,0x431e2314aab9531cu64,0x30486ca520d654adu64,0x2c3264b433668860u64,0xf266f8667da2ea75u64,0xe543194e96996b5au64,0x15360c94732a981fu64,0xeb397b3197880fe0u64,0xca0f25128cd53924u64,0x9ec14554123a3751u64,0x93f8107d5542f30fu64,0xdb2935eb927c645cu64,0x92095eb737e71abcu64,0x3e074548d079d270u64,0x288328e01018ec43u64,0x9b7f579eab6c5f50u64,0x2741bf3eadeb9babu64,0x9376987180b45578u64,0x5ca3729a00302f4eu64,0xe2174c3472c699c8u64,0xe471b21163191459u64,0x6366a6b75b379303u64,0x92117030488f63e4u64,0x2e43c29c546806bbu64,0xc5f69e4c1342783eu64,0x4d089fe5c9860302u64,0x85336d51fe6ac80cu64,0x38a52a39467ee92eu64,0xff001fdab747878u64,0x56b0dfecead63afu64,0xf5a11c742726fa31u64,0x7d60a4c5dbad8f15u64,0xcb4571cabddaa5dfu64,0xefd19b8d708d1697u64,0x4695238d23c84197u64,0x370cd82d6eac33e4u64,0xd66e12ef9f69591au64,0xac5c92dd66d874d2u64,0x397eb2251ccef258u64,0x3ead39d2b764a9au64,0xafa9c4649ad9d52u64,0xf1c42762e30a7832u64,0xb5011cd278c5ae2u64,0x3cc969a42557c1f6u64,0xbfe38b2457a00005u64,0x5b4f19d66b42bd7du64,],[0xbf398a76af28d0b2u64,0x830123f08725adcdu64,0x39345dd49cd3e3c8u64,0xff562b16877fcc58u64,0xe3315ee16333741bu64,0x4f4ba7ffcc726569u64,0xd56135e5de2dabu64,0x3b2e580f98b5f11u64,0xb125a4183bdbd948u64,0x52e5e57e240c41a0u64,0xc3d486662734c54cu64,0x935994ac0e5a46f6u64,0x4ca72b395329aff2u64,0xa75dc68ba23c0817u64,0x399ddcafdca1ba4eu64,0x8b61c81f8222e8edu64,0x1ecb4bd4b2bf32fcu64,0xb0c684d4a40afbe3u64,0xd72e8e8797f17634u64,0xbaecf5f73c8b9327u64,0x6a89fe5da3c71e2u64,0x128eaa5f9567a36du64,0xa3b69dd662ff9e78u64,0x1eaadb53598c31bbu64,0xac42b7a2453ed8b8u64,0xec3cc327710a97acu64,0xd839eb6ef97cbd8bu64,0x1ed1932d4097afb7u64,0x1bfe9f0377efa329u64,0x2c317b9696ab77du64,0x33e2893fe1259ec5u64,0x97a7d76ce81a401bu64,0xc907c694fcc92091u64,0xd9a4d7c4aa495c61u64,0xdfc3ca5b91307ecbu64,0xf544240d5c96c0b5u64,0x66763dcb84e29795u64,0xba7dce42ab479cb6u64,0x88b1800598108939u64,0xc69c641df7fdd69fu64,0x8762db31fa379899u64,0xf9e24682b8525cf8u64,0xaecd6eb249582771u64,0x2580992a4a3ae964u64,0x917b4cc4edbf66eeu64,0x46462c8f7d3b5ddcu64,0xa1b1ed9cc9118e87u64,0x7507f008b1983e56u64,0x14a5e3c87ecf2542u64,0x2f9bb9f6c433b392u64,0xa41dd125c60a6904u64,0x5f48649675a045abu64,0x18a6d3e0197aafd4u64,0x134542dd789cb7e3u64,0x167a2cc8fcc6f000u64,0x82df63bcaae326feu64,0x12c5fedbba26031bu64,0x6324cda2ccf79965u64,0x309dca29ffd28287u64,0x777c076fa8c350c8u64,0x9dbadd9d68fabf17u64,0x8d7fbc1b695f8d4bu64,0x4ca3fc46ab78ac48u64,0x224e25207d8467acu64,],[0x6c87c47f6c3f581au64,0x145ecc63ccd1ebdu64,0x761d2296bf403102u64,0x5ef41cfa74367b38u64,0x762f3f24cc98b23cu64,0x9e9d39d47372c6f0u64,0x849c789592c1e19bu64,0x638ec62effed44a7u64,0x39a802506109af60u64,0x89802e0ba75d6d14u64,0x8013445b3bda8955u64,0xb8b3e507be0f2dau64,0x970d05636dabee04u64,0xb4c9c2115a6cd5f1u64,0xcf34728e8e868d45u64,0x3f00788d208041bu64,0xb916022433b255b0u64,0x67d5922c129d7b0au64,0x52236711b6c7a7d5u64,0x9559359aafb3bcebu64,0x9ad074e4c4dd89cbu64,0xe9960ea7d8698a9au64,0xd29549b6b9b66dffu64,0x49d0f4b362cafc7cu64,0x1def14046c47f4c7u64,0x7cfa5ea5cde8b293u64,0xe8113f7e00410b20u64,0xe38136a0f2bca5bfu64,0x944d21d430eb2dddu64,0xde277ad0ca97f86fu64,0xbdc4c808643865aau64,0x990dcba2397c6b80u64,0x850ec78dcf38d8a1u64,0xea5c14d87d7238d9u64,0x520a6adbff59708cu64,0x68da79503d6165aeu64,0x82a6d22b81090618u64,0xabb6c8b072520276u64,0x594fdb080d4360e5u64,0xa38725e783e14a65u64,0x3fe559407ed8f47au64,0xe8afbe124b0d35du64,0x639993e0e3a83102u64,0xf3b66e596c479b71u64,0xb2e3b1fe578e661eu64,0x2bfe4cf6b3a1b26cu64,0x327041f70f0b40b7u64,0x1f75617d925ead61u64,0xf24b05700fd00549u64,0xd4d4ed1007197e16u64,0x8b599a51fbfa58d1u64,0x1827b05125614efu64,0xcf5309ba9b8c014au64,0xd597ac9950def07fu64,0x6045405865b47dd0u64,0xded5bc9c91ca5e21u64,0xa8869de990ea19dbu64,0x773beab07594bcf6u64,0x872ad2c952b93cb5u64,0x365a912bd033dd40u64,0x8514fca8a5f68733u64,0xcdff514e13862353u64,0xb145e9a8ba1d5e4cu64,0xe60a9e8e71aa1026u64,],[0x8b4bc41e567bee1fu64,0x920da66b8f076a60u64,0xbc850d66393d94e4u64,0x3dd4930c3bfbffedu64,0x16384f0652358bf1u64,0x5491023bbfe5ce23u64,0x22089681ea2b99e3u64,0xdb6d0908402b7bd3u64,0x2cdbd3f2047aaed9u64,0x90f50188f05915b4u64,0x916bf99032593714u64,0x8c22a2937a8149f4u64,0x2d52052a7bcded37u64,0x1957e6469d778ca8u64,0xe0c4cf86f80a32e1u64,0xc483106dd63b008fu64,0x9119bddd9cf67u64,0x6768230a1a8139ffu64,0x7bcbbb0a8587f9c2u64,0x9cd21fb04c3027f4u64,0xc26ec52d24c80dc8u64,0xedf6dd946eecb011u64,0x4ba0e1791bb1acb7u64,0x2c8af95a1aa2faa2u64,0xd211dee70eb980a7u64,0xaa64414e41a6b1f5u64,0x525d9d760658e704u64,0xaa815b67c84bf56eu64,0x1f550bbb5dc3866au64,0xb5345d160f9c5f7fu64,0xc063d67aed703000u64,0x546c91bce621885du64,0x9f1f54c5d1c9763bu64,0x4703200ce07030e9u64,0xf205cb1f0a2d0eb5u64,0x3dbe8cb23a61200fu64,0xaf5e511d0fd8c7eu64,0x9c488aaab3abfdefu64,0x85bb52480b362617u64,0xada512c62f175bau64,0xe888e991af37f34bu64,0xb00834ca01c970f4u64,0x54481464c5d5cd75u64,0xbba93e2ec67d5d1au64,0x1dfb4bde81e49d48u64,0xb09497100f82df44u64,0x42707fe2fa0745e6u64,0xc1c2695037178e8bu64,0xb98d53217b9fd3efu64,0x7c7c33029661b50cu64,0x457d23245cbf0468u64,0x3afd700c5db692acu64,0xae776655c7ac9471u64,0x5a92bbf5446db43du64,0x38600e8044555bc1u64,0xf3072c6a54b66c65u64,0xe42047e2a11739b0u64,0x3e1e2e91886e6b92u64,0x83170f7d879b701u64,0xcfce74f7e7e760b6u64,0x1ca3a1db64f80347u64,0x8f7e4feb06e0e3f4u64,0x4cea8b036405bb5du64,0xbb6a81a049630ae4u64,],],[[0x6c81f00508572f83u64,0xdc95f580b4180b5cu64,0xf8c8c63553ca00c0u64,0x9e53a13c92512963u64,0x918738eb9464ee0cu64,0xf3cc83f55c64e626u64,0xa200382ecdd29c68u64,0x8903d71374fc7e0fu64,0x27f2c7b3227e52d8u64,0x3e9692e5c103a326u64,0xb499015955092761u64,0x324d299984ce3d43u64,0x6fe56412a74bea51u64,0xc004e77cfba7bd01u64,0xd0e42bcd5253ad86u64,0xa62bae577ec671eeu64,0x1b9920c34d664311u64,0x1fcf235162be13fdu64,0x2ab2fb634f47c72au64,0x92977bd6e20d66f1u64,0x34302782542a7830u64,0x542fa6e09b4042deu64,0x9e674ec8383ddf52u64,0xa354691d459d2da4u64,0xb240f1c5ecb8cb0du64,0xa0c08dc2e8e3f19du64,0x7ab93b391c403dbbu64,0x6586af574a17010fu64,0x72299b4cf1dfbe64u64,0x7547624d7731a1beu64,0xbdb16e44e02d2737u64,0x1445d0b28f05c025u64,0x5641c061c3aa8600u64,0xf9dec385fedfa9cu64,0x7027fc5a05b51ea3u64,0x257a677ca28ad438u64,0x1190deb178ae1afeu64,0xd2b3a036d34f2743u64,0x704dd0c763bc6e4au64,0x5f74100307875354u64,0x95597be8de9aa794u64,0x1a83090eadcc31bdu64,0x37fbdc53fdfb03c4u64,0x9bd64defa0df7df3u64,0x676bb1db6269b072u64,0x79db2c1d0efcd7fau64,0x1b7c43aee4ef0742u64,0xbdb2c41c7e9e104fu64,0x281c6419284d195cu64,0x5ecf406464d13527u64,0x6fe730ec4f4fd39eu64,0x3fe412d85b21061cu64,0x7cc74f221c822253u64,0x312adb8984a12b58u64,0x98e90104608c6fecu64,0x7e0f3aba3499b8e0u64,0x13276ec58eaba21du64,0xf013f15ffdf49d04u64,0x98d161824cb2a5c6u64,0xa941d6c979bae5aau64,0x62264c7fe17f22fcu64,0x79e3b3d64b8458dau64,0x284e7d7535b07c7cu64,0x8a40dadd7d3b645du64,],[0x1a08d3483c5908a5u64,0x4fadf13ba87907dbu64,0x1464c39fa368ed9u64,0x603914d89d4338f1u64,0xc45665486b6f0095u64,0x59e92680b1c48ca3u64,0x2f778366b89bffabu64,0xad96976ccfeb515eu64,0xbbc9d2dcaa43aa34u64,0x38456c0184035f1du64,0x7a5ba7d58a387e7au64,0x461dbb42de93979fu64,0xabab6c38c72a9040u64,0x3c829f07fbea3a73u64,0xd6d1b2dddff8d590u64,0x82f540e5378ad798u64,0x7e2a296240d2e065u64,0x710f9e1ae4306f76u64,0xad5978827e28a49du64,0xd9a7eb32251250bdu64,0x513c191a84b85733u64,0x9162d581fc2e5a3eu64,0xc365fa57aa43d9bau64,0x6071642e3756ec25u64,0x66e10fa4815ca47u64,0x577242caa9160b92u64,0x1c78c4c2d94bdb5eu64,0xb30675a09f0cc87u64,0x981668d8611b748u64,0x46c5fbd5301eb0d5u64,0xcab4b28fb9da213au64,0x981ac886123255dfu64,0x7af5098b2dc3cc6bu64,0x16f0c63757b09cceu64,0x586565d5bf9a5f7fu64,0x5202e10d1e710d4cu64,0xbadd23dc772a96c2u64,0xa1eefefc3169d809u64,0xb89924b789210cd4u64,0x70681be38725b31fu64,0x183e5be8d536295du64,0x5834da331181c13fu64,0xf82ff2fdb62580d3u64,0x8cd804fc837c215au64,0xbbca88a950ce140du64,0x46cd5cdfd71b6dd7u64,0x5ecda29e686c447u64,0x670e6e13e979c027u64,0xea2b6f0882cfb05bu64,0xbab41f55560115c9u64,0x7ac445e571686d7eu64,0x1ae63cfb911dacaeu64,0x321dc03b0acb3438u64,0xf397ea560a5f0dcu64,0xc7a04760f3ad8623u64,0x552f614a404973cu64,0xe2328b0f93cadc8du64,0xdefbc6694b8ba94eu64,0xd1a2573fe0f8c94cu64,0x41017684262625f5u64,0xdfa59ba2ecb60641u64,0x9334eb087b8684ccu64,0xd84797cdf3c56ab2u64,0xbbce281e6f460386u64,],[0x9282e62173954981u64,0x27df51117d4c4b9cu64,0x672ae6be4d7bba52u64,0xb19b0d972ace8f43u64,0x73d33bcd1fa4750eu64,0x2d96734e545d2942u64,0x764cd2db9c2fdb37u64,0x530830ae8ccf4f88u64,0x7b064968232c7f13u64,0xcb64f843eea28636u64,0xdfe373a8f7ffc8cfu64,0x13426905be27316cu64,0x7a6c091fe810dad1u64,0x8057ee23c13dfed5u64,0x4ee90d5dd5b40800u64,0xf2111ac4bd816451u64,0x1aa39a2400170858u64,0x34c19867e591d731u64,0x4f0980d76381c7d8u64,0x1a1b68514d098c94u64,0xb250dfc26ef65c0du64,0x3c65492b6fb95bc1u64,0x7b25047d3c5a1e4cu64,0xef5622dfa234ccau64,0x97c01816ef5276b8u64,0x2f88f528ff699e43u64,0xeab60a19c0dd2a2u64,0x51bd4e7edf9c71ecu64,0xc0df267e22be1cd7u64,0x7e21ad92411ad377u64,0x61ae67af5ccc4621u64,0x5c9b2c805832a3f9u64,0xe2377f2b2f18f023u64,0x6a253ef52b7e3571u64,0xf845979d85068753u64,0xb3c882be35bdf470u64,0xcbddded935c6c01eu64,0x2933884573a2f04au64,0xe4540d37da94936cu64,0xb5ada38271b6c03u64,0xa8353a9ae0c82bau64,0x28a2b7bf697330ccu64,0x22f2fe9f3f65257bu64,0xa1f2b9940266d706u64,0xb92808057087a34eu64,0x8fd33fe0d39749bcu64,0x641c32ba0b77b47bu64,0x9db4d182abdb77e4u64,0xf1bf3052519d9779u64,0xdaab10e4cde142b0u64,0xa0053a5a08fc107du64,0x5403e9b439392febu64,0x5ab2a0fdd93be8e7u64,0x56cf700fd251dcfcu64,0xa3ef47a25ff266dfu64,0x57badca09e28d2aeu64,0x441b2eb88e661cb6u64,0x14d9e687e16c98aeu64,0x1224170219a65c73u64,0xa6a105bc0c539bb4u64,0x1e670308035e2bc5u64,0xbc29c9adea1e209bu64,0xb31a4de38763fa6u64,0x5569f7028bd278a6u64,],[0x96f3239ed2d65a0au64,0x8017f78e4cfe8533u64,0xc976a57dab71ec14u64,0xe842625079258ab4u64,0x6a29196bbe611831u64,0x93d13b99bd196946u64,0x1c07c046425dcd19u64,0xb8de738110d9dcb7u64,0x1bf046096d601d2eu64,0x4ba8515d0040651u64,0x5a3884f543e15f41u64,0x1891e045b4abc2d6u64,0xb3ebd4b4139d3ca2u64,0xea83aa9c78feda6bu64,0xa96259c8cab91509u64,0x235a1e47fd336f8u64,0xf7bfbc08ed09116au64,0x33e28ad510f6befu64,0x25f78b0e861d3f95u64,0xe9694be612ee9decu64,0xadd84079235ed14eu64,0xbcc9163c5e73c002u64,0x5f251be44c35d62bu64,0x612c0be83fc9300eu64,0x98d8c9d7c7189debu64,0xbc5a0e1b1ec1b710u64,0xb864c0066bc1ca9au64,0x6c4fc0544a6b9aa9u64,0xf2cd43e474ac7370u64,0x2f41899c1a1f4f1cu64,0x3cfe3ad528ff6311u64,0xe155444a0a7068eeu64,0xf6d04d1d1e53ee16u64,0x87fc6befb5d1ffc8u64,0xd9d4e1d11cdf0096u64,0x939defeef32e29b3u64,0xe505ad551cc468f7u64,0x50b9533e72745736u64,0xe2da65fb34c069c2u64,0xd3a385e0af66c6c4u64,0x5f443e147cc75cd6u64,0xe32be0ed290e94eu64,0xdb12569b67adbce9u64,0x2e91a0c2bc2575fau64,0x83b6e72517b614eu64,0x8535af247e3e7471u64,0x82777b46c704e038u64,0xfcc8cbe4c2ccbab1u64,0xd4c57f6f9204d4b5u64,0xff9d84286b4715adu64,0xf9867d7b251d8fe3u64,0x5d1ff46aa0513098u64,0x7aa06490d1fe4cebu64,0x64228622d59fb22bu64,0xa24ae70f8b514efbu64,0xaa9e5d3fcd6258fdu64,0xe348e96ddd9a0eecu64,0xdedefc83cd696874u64,0xdb16335e1d75dc7fu64,0x94869b47d33ba8deu64,0xbf9384292f6b4de1u64,0x7fa6b10cc262ec76u64,0x450b0c082ffcf3bau64,0xaa510294379c50ffu64,],[0x1a2ed90ecadc23cfu64,0xb64c70350fd42792u64,0x571159c01bd55b0au64,0x33f59f5a11e5477eu64,0xf2436c2971322923u64,0x98ac0e11e34ac192u64,0x140a0cddac3eea9bu64,0x3ed8e27fe3f7a106u64,0x1fcc846489bf61bcu64,0x4387f1374ae9f42au64,0x997e8007ed0e4730u64,0x3418148fc6f2661cu64,0x3136b4e4ce59224u64,0xd678ae45243593f3u64,0x8bf8d21e470409d1u64,0xa17002d3c49f59cdu64,0x666f694a68a630b0u64,0x389156bc0b6454d6u64,0x7f85f32d587db526u64,0xb49391d63869d84au64,0x283f463d5f49dddau64,0xfe789b70fc540db4u64,0x4c4035d2dc5b119fu64,0xacde3b491827bca2u64,0x23654435d1e9c44fu64,0x6b04fcc6b89b672cu64,0x2620746322dc587au64,0x1c9212f06f2607b0u64,0x31618b98012b6e23u64,0x7dd9b0c8b669ef6au64,0xb01713a8c97fcf5bu64,0xc315d28ccd1ed9f4u64,0x1badaf4511afc3b7u64,0xc9e17faa84d0a527u64,0xb3090c6a45f8543du64,0x59c1e2e0acdf7c15u64,0xfa0224d7a8c9d0ceu64,0x1a8076b1aadd9cf6u64,0xb5c58d4c6f3462aau64,0x2e6d8726da6df49bu64,0xf227cf90ac6cef0cu64,0x8a1113a7640bdc3cu64,0xf87d872d495180c9u64,0xb334b2ac47650c0u64,0xcd3b13fc3c3c123cu64,0xcd2315959ee157eau64,0x8d350716146b99f5u64,0x979b24846c12c40du64,0x2bf553af0e61d28bu64,0x66283e0f46df69d6u64,0x79a972b233ab7383u64,0xe491a6d98ac72be5u64,0xac0b734c596d8f0cu64,0x5673f834a3c729du64,0xfc8dfa8745ebacf8u64,0xdc1947355d1db55u64,0xe02420156cc83d5fu64,0x7579d4ff0924f3c1u64,0x25a707a2dcca6c76u64,0x4694ba8fe0e7ebfau64,0x4ad67882528376ceu64,0x3b301f484a282d7eu64,0xc345785ff2df2ab2u64,0x6ec36e79d2c470feu64,],[0x7348ab9c631cb678u64,0xc5b55ef1305ca607u64,0xfdf7c613071acef3u64,0xd4f5e5055d352514u64,0x71fe33ed1e4d449au64,0x26313ed41ffc2d40u64,0x9e37aa8b025864edu64,0xebb3bc76f7d9b0e6u64,0x5a52c35d51ae1423u64,0x9467afd658a1cac9u64,0x9d0ed35eb9d10ec4u64,0x21339189365d4708u64,0x5f6c3b61dacfa4c5u64,0x1f45b6e77e480eabu64,0x142d076b51f64d5eu64,0xe9b028b5a470711fu64,0x9ebd08c36d27778u64,0xd7d7ed05694862dfu64,0x50b0c0600fa8c966u64,0x16b07b1eab6b11f8u64,0x4dbdaeedd1b18238u64,0xffb88915d6457fa4u64,0x14e70b66e27887f5u64,0xcffc98d4c48a0c22u64,0x9f0e4ea3c0a889e5u64,0xeebec8d880624e3du64,0xd27954e4194ffe7cu64,0xe7372ad6c32cec4eu64,0x5404d9b2f8dc0920u64,0x8624b975967d5020u64,0xcce16e6b8d22da1du64,0xfd05363b5df0ad16u64,0x8edc7da89f99467fu64,0x7465dcc9976402e5u64,0x251fe0af591db19du64,0xb28490313da71251u64,0x6ec046f32713b3cfu64,0xef77e9ef36f0ff98u64,0xaeab80eb8a01ed4du64,0x6d2979f50940f67u64,0x71174320067198e3u64,0x769262537ee53614u64,0xd461f6f3fda8af3du64,0x3f0c0a8cfcfd6e1cu64,0x6af9dbf9d1c99f4au64,0x2a66c541c3e5b0c3u64,0x1fa9866b8e9e763au64,0x157abe8ba35ade5du64,0x38ac8e2a8b819a5fu64,0x68aa95c278cd829eu64,0xb70e2a8fd69f08fcu64,0x872ee374150a3f7bu64,0x150c3eb25bc1b5dcu64,0x48566aa01e1c9abeu64,0xfe0147bbcdd3445fu64,0x21e392b84d898f69u64,0xe35b1575aa7212fcu64,0x696390e33969dbd0u64,0xf6d19ccaa8c635b9u64,0xce1326504814fb14u64,0xeb7d7e3125600602u64,0xd1c21e4fd61c9b2fu64,0xe41eb3ecf593e4bu64,0xf20834db69dce8eu64,],],]
};
pub fn rand_array_64(rng: &mut StdRng) -> [u64; 64] {
    let mut res = [0u64; 64];
    for item in res.iter_mut() {
        *item = rand_u64(rng);
    }
    res
}

pub fn rand_array_16(rng: &mut StdRng) -> [u64; 16] {
    let mut res = [0u64; 16];
    for item in res.iter_mut() {
        *item = rand_u64(rng);
    }
    res
}

pub fn rand_array_8(rng: &mut StdRng) -> [u64; 8] {
    let mut res = [0u64; 8];
    for item in res.iter_mut() {
        *item = rand_u64(rng);
    }
    res
}

pub fn init_zobrist() -> Zobrist {
    let mut generator: StdRng = SeedableRng::from_seed([42; 32]);
    let mut pieces = [[[0u64; 64]; 6]; 2];
    for side in 0..2 {
        for pt in [
            PieceType::Pawn,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Queen,
            PieceType::King,
        ]
        .iter()
        {
            pieces[side][*pt as usize] = rand_array_64(&mut generator);
        }
    }
    let side_to_move = rand_u64(&mut generator);
    let castle_white_kingside = rand_u64(&mut generator);
    let castle_white_queenside = rand_u64(&mut generator);
    let castle_black_kingside = rand_u64(&mut generator);
    let castle_black_queenside = rand_u64(&mut generator);
    let mut castle_permissions = [0u64; 16];
    for i in 0..16 {
        if i & CASTLE_WHITE_KS > 0 {
            castle_permissions[i as usize] ^= castle_white_kingside;
        }
        if i & CASTLE_WHITE_QS > 0 {
            castle_permissions[i as usize] ^= castle_white_queenside;
        }
        if i & CASTLE_BLACK_KS > 0 {
            castle_permissions[i as usize] ^= castle_black_kingside;
        }
        if i & CASTLE_BLACK_QS > 0 {
            castle_permissions[i as usize] ^= castle_black_queenside;
        }
    }
    let keys = Zobrist {
        pieces,
        side_to_move,
        en_passant: rand_array_8(&mut generator),
        castle_permissions,
    };
    keys
}

pub struct Zobrist {
    pub pieces: [[[u64; 64]; 6]; 2],
    pub side_to_move: u64,
    pub castle_permissions: [u64; 16],
    pub en_passant: [u64; 8],
}
impl Zobrist {
    fn piece_string(&self) -> String {
        let mut res_str = String::new();
        res_str.push_str("[");
        for side in 0..2 {
            res_str.push_str("[");
            for pt in 0..6 {
                res_str.push_str("[");
                for sq in 0..64 {
                    res_str.push_str(&format!("0x{:x}u64,", self.pieces[side][pt][sq]));
                }
                res_str.push_str("],");
            }
            res_str.push_str("],");
        }
        res_str.push_str("]");
        res_str
    }
}
impl Display for Zobrist {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut res_str = String::new();
        res_str.push_str("#[rustfmt::skip]\n");
        res_str.push_str("pub const ZOBRIST_KEYS : Zobrist = Zobrist{\n");
        res_str.push_str(&format!("side_to_move: 0x{:x}u64,\n", self.side_to_move));
        res_str.push_str(&format!(
            "castle_permissions: {:?},\n",
            self.castle_permissions
        ));
        res_str.push_str(&format!("en_passant: {:?},\n", self.en_passant));
        res_str.push_str(&format!("pieces: {}\n", self.piece_string()));
        res_str.push_str("};");
        write!(f, "{}", res_str)
    }
}
