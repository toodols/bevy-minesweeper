document.body.innerHTML = "<canvas id='c' width='32'></canvas>"
$("#c").height = 32*12
const ctx = $("#c").getContext("2d");
function background(s) {ctx.fillStyle =s; ctx.fillRect(0, 0, 32, 32*12)}
function clear(){background("white")}
clear()
background("#aaa")
ctx.font = "20px sans-serif"
ctx.textAlign = "center"
ctx.textBaseline = "middle"
for (let i = 0; i<=11; i++) {
    if (i === 0){
        ctx.strokeRect(0, -1, 32, 32)
    }
    if (i === 9) {
        ctx.fillStyle = "red"
        ctx.fillRect(16-3, i*32+16-3,6,6)  
    }
    if (i=== 11){
        ctx.fillStyle = "#888"
        ctx.fillRect(2, i*32+16-14, 32-4,32-4)
    }
    if (i > 0 && i < 9) {
        ctx.fillStyle = ["red","green","blue", "purple", "yellow", "orange", "black", "teal"][i-1]
        ctx.fillText(""+i, 16, i*32+16)   
    }
}