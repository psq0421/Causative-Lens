import {test} from 'node:test';
import assert from 'node:assert/strict';
import {filterRows,counts,appendPage,windowRange} from '../../crates/clens-gui/frontend/model.mjs';
const rows=[
{seq:1,category:'assertions',source:'assertion',target:'config.yaml',text:'Read'},
{seq:2,category:'files',source:'agent',target:'config.yaml',text:'<script>alert(1)</script>'},
{seq:3,category:'network',source:'agent',target:'api.example.com',text:'Connect'},
{seq:4,category:'other',source:'agent',text:'hello'}
];
test('four filters select real distinct records and search combines with filter',()=>{
 assert.equal(filterRows(rows,'all','').length,4);
 assert.deepEqual(filterRows(rows,'assertions','').map(r=>r.seq),[1]);
 assert.deepEqual(filterRows(rows,'files','CONFIG').map(r=>r.seq),[2]);
 assert.deepEqual(filterRows(rows,'network','api').map(r=>r.seq),[3]);
 assert.equal(filterRows(rows,'files','missing').length,0);
 assert.deepEqual(counts(rows),{all:4,assertions:1,files:1,network:1});
});
test('cursor pages deduplicate, preserve monotonic order and bound memory',()=>{
 let state=appendPage([],rows);
 assert.equal(appendPage(state,rows).length,4);
 state=appendPage(state,Array.from({length:6000},(_,i)=>({seq:i+5,category:'other'})));
 assert.equal(state.length,5000);assert.equal(state.at(-1).seq,6004);
});
test('100k rows remain windowed and empty windows stay valid',()=>{
 const data=Array.from({length:100000},(_,i)=>({seq:i,category:i%2?'files':'network',target:'x'}));
 const start=performance.now();
 assert.equal(filterRows(data,'files','x').length,50000);
 const elapsed=performance.now()-start;
 const range=windowRange(100000,350000,700,56);
 assert.ok(range.end-range.start<=500);
 assert.ok(range.start>=0&&range.end<=100000);
 assert.deepEqual(windowRange(0,0,700,56),{start:0,end:0,top:0,bottom:0});
 console.log(JSON.stringify({benchmark:'filter-100k',milliseconds:elapsed}));
});
