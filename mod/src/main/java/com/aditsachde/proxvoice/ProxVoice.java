package com.aditsachde.proxvoice;

import net.minecraft.entity.player.EntityPlayer;
import net.minecraft.util.math.AxisAlignedBB;
import net.minecraft.util.math.BlockPos;
import net.minecraftforge.common.config.Config;
import net.minecraftforge.common.config.Config.Type;
import net.minecraftforge.fml.common.FMLCommonHandler;
import net.minecraftforge.fml.common.Mod;
import net.minecraftforge.fml.common.Mod.EventHandler;
import net.minecraftforge.fml.common.event.FMLInitializationEvent;
import net.minecraftforge.fml.common.event.FMLPreInitializationEvent;
import net.minecraftforge.fml.common.eventhandler.SubscribeEvent;
import net.minecraftforge.fml.common.gameevent.TickEvent;
import org.apache.logging.log4j.Logger;
import org.zeromq.SocketType;
import org.zeromq.ZContext;
import org.zeromq.ZMQ;

import java.util.List;


@Mod(modid = ProxVoice.MODID, name = ProxVoice.NAME, version = ProxVoice.VERSION, clientSideOnly = true, acceptedMinecraftVersions = "1.12.2")
public class ProxVoice {
    public static final String MODID = "proxvoice";
    public static final String NAME = "ProxVoice";
    public static final String VERSION = "0.1";

    private static Logger logger;
    private int ticks;
    private ZMQ.Socket socket;

    @EventHandler
    public void preInit(FMLPreInitializationEvent event) {
        logger = event.getModLog();
    }

    @EventHandler
    public void init(FMLInitializationEvent event) {
        FMLCommonHandler.instance().bus().register(this);
        ticks = 0;
        ZContext context = new ZContext();
        socket = context.createSocket(SocketType.PUB);
        socket.bind("tcp://127.0.0.1:" + CONFIG.port);
        logger.info("Initialized ProxVoice!");
    }

    @SubscribeEvent
    public void tickEvent(TickEvent.PlayerTickEvent event) {
        ticks++;
        if (ticks >= CONFIG.ticks) {
            ticks = 0;
        }
        if (ticks == 0) {

            EntityPlayer player = event.player;
            AxisAlignedBB bb = (new AxisAlignedBB(new BlockPos(player)))
                    .expand(CONFIG.radius, CONFIG.radius, CONFIG.radius)
                    .expand(-CONFIG.radius, -CONFIG.radius, -CONFIG.radius);
            List<EntityPlayer> players = player.world.getEntitiesWithinAABB(EntityPlayer.class, bb);

            for (EntityPlayer otherPlayer : players) {
                int distance = (int) player.getDistance(otherPlayer);
                String send = otherPlayer.getUniqueID().toString().replace("-", "") + " " + distance;

                try {
                    socket.send(send);
                } catch (Exception e) {
                    System.out.println(e.toString());
                }
            }

            try {
                socket.send("frame");
            } catch (Exception e) {
                System.out.println(e.toString());
            }
        }
    }

    @Config(modid = MODID, type = Type.INSTANCE)
    public static class CONFIG {
        @Config.RequiresWorldRestart
        @Config.RangeInt(min = 40000, max = 50000)
        public static int port = 40544;

        @Config.RangeInt(min = 10, max = 200)
        public static int ticks = 30;

        @Config.RangeInt(min = 5, max = 300)
        public static int radius = 32;
    }

}
