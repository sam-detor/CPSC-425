/*
 * i2s-beep.c
 *
 * author: Furkan Cayci
 * description:
 *   talk to CS43L22 Audio DAC over I2C
 *   connected to I2C1 PB6, PB9
 *   setups CS43L22 to play 6 beep sounds from headphone jack
 *   Audio out is connected to I2S3
 */

#include "stm32f4xx.h"
#include "system_stm32f4xx.h"
#include "cs43l22.h"

#define C6 (1 << 7)
#define D6 (0x90)
#define E6 (0xA0)
#define F6 (0xB0)
#define G6 (0xC0)
#define A6 (0xD0)
#define B6 (0xE0)

#define C5 (1 << 4)
#define D5 (1 << 5)
#define E5 ((1 << 4) | (1 << 5))
#define F5 (1 << 6)
#define G5 ((1 << 6) | (1 << 4))
#define A5 ((1 << 6) | (1 << 5))
#define B5 ((1 << 6) | (1 << 5) | (1 << 4))
#define Rest (0x0)
#define TWINKLE_LEN (48)
#define MARY_LEN (31)
#define TEMPO_86 (0)
#define TEMO_430 (1)
#define TEMPO_780 (2)
#define TWINKLE (0)
#define MARY (1)

//systik stuff
static volatile uint32_t tDelay;
extern uint32_t SystemCoreClock;

uint32_t playing = 1;
uint32_t clicks = 0;
uint32_t twinkle_note = 0;
uint32_t mary_note = 0;
uint32_t songChoice = MARY;
uint32_t mary_speed = 1;
uint32_t twinkle_speed = 1;

uint8_t twinkle[48] = {C5,C5,G5,G5,A5,A5,G5,Rest, 
                      F5,F5,E5,E5,D5,D5,C5, Rest, 
                      G5,G5,F5,F5,E5,E5,D5, Rest,
                      G5,G5,F5,F5,E5,E5,D5, Rest,
                      C5,C5,G5,G5,A5,A5,G5, Rest,
                      F5,F5,E5,E5,D5,D5,C5, Rest};
uint8_t twinkleTempo[48] = {0,0,0,0,0,0,1,0, 
                            0,0,0,0,0,0,1,0, 
                            0,0,0,0,0,0,1,0,
                            0,0,0,0,0,0,1,0,
                            0,0,0,0,0,0,1,0,
                            0,0,0,0,0,0,1,0};
uint8_t mary[31] = {E5,D5,C5,D5,E5,E5,E5,Rest,
                   D5,D5,D5,Rest,E5,G5,G5,Rest,
                   E5,D5,C5,D5,E5,E5,E5,
                   C5,D5,D5,E5,D5,C5,Rest};

uint8_t maryTempo[31] = {1,1,1,1,0,0,1,0,
                        0,0,1,0,0,0,1,0,
                        1,1,1,1,0,0,1,
                        1,0,0,1,1,1,0};

uint32_t note = 0;
uint32_t len = MARY_LEN;
uint8_t *song = mary;
uint8_t *tempo = maryTempo;
uint32_t speed = 1;

enum processState {NO_PRESS, FIRST_PRESS, FIRST_PRESS_DEBOUNCE, FIRST_UNPRESS, FIRST_UNPRESS_DEBOUNCE, FIRST_PRESS_DONE, SECOND_PRESS, SECOND_PRESS_DEBOUNCE, LONG_PRESS, LONG_PRESS_DONE};
enum processState myState = NO_PRESS;

/*************************************************
* function declarations
*************************************************/
int main(void);
void init_i2s_pll();
void init_i2s3();
void init_cs43l22(uint8_t);
void start_cs43l22();
void init_systick(uint32_t s);
void delay_ms(uint32_t);
uint32_t getDelay (uint32_t speed);
uint32_t getTempo (uint8_t tempo, uint32_t speed);

/*************************************************
* I2C related general functions
*************************************************/
volatile uint8_t DeviceAddr = CS43L22_ADDRESS;

static inline void __i2c_start() {
    I2C1->CR1 |= I2C_CR1_START;
    while(!(I2C1->SR1 & I2C_SR1_SB));
}

static inline void __i2c_stop() {
    I2C1->CR1 |= I2C_CR1_STOP;
    while(!(I2C1->SR2 & I2C_SR2_BUSY));
}

void i2c_write(uint8_t regaddr, uint8_t data) {
    // send start condition
    __i2c_start();

    // send chipaddr in write mode
    // wait until address is sent
    I2C1->DR = DeviceAddr;
    while (!(I2C1->SR1 & I2C_SR1_ADDR));
    // dummy read to clear flags
    (void)I2C1->SR2; // clear addr condition

    // send MAP byte with auto increment off
    // wait until byte transfer complete (BTF)
    I2C1->DR = regaddr;
    while (!(I2C1->SR1 & I2C_SR1_BTF));

    // send data
    // wait until byte transfer complete
    I2C1->DR = data;
    while (!(I2C1->SR1 & I2C_SR1_BTF));

    // send stop condition
    __i2c_stop();
}

uint8_t i2c_read(uint8_t regaddr) {
    uint8_t reg;

    // send start condition
    __i2c_start();

    // send chipaddr in write mode
    // wait until address is sent
    I2C1->DR = DeviceAddr;
    while (!(I2C1->SR1 & I2C_SR1_ADDR));
    // dummy read to clear flags
    (void)I2C1->SR2; // clear addr condition

    // send MAP byte with auto increment off
    // wait until byte transfer complete (BTF)
    I2C1->DR = regaddr;
    while (!(I2C1->SR1 & I2C_SR1_BTF));

    // restart transmission by sending stop & start
    __i2c_stop();
    __i2c_start();

    // send chipaddr in read mode. LSB is 1
    // wait until address is sent
    I2C1->DR = DeviceAddr | 0x01; // read
    while (!(I2C1->SR1 & I2C_SR1_ADDR));
    // dummy read to clear flags
    (void)I2C1->SR2; // clear addr condition

    // wait until receive buffer is not empty
    while (!(I2C1->SR1 & I2C_SR1_RXNE));
    // read content
    reg = (uint8_t)I2C1->DR;

    // send stop condition
    __i2c_stop();

    return reg;
}

/*************************************************
* I2S related functions
*************************************************/

// enable I2S pll
void init_i2s_pll() {
    // enable PLL I2S for 48khz Fs
    // for VCO = 1Mhz (8Mhz / M = 8Mhz / 8)
    // I2SxCLK = VCO x N / R
    // for N = 258, R = 3 => I2SxCLK = 86Mhz
    RCC->PLLI2SCFGR |= (258 << 6); // N value = 258
    RCC->PLLI2SCFGR |= (3 << 28); // R value = 3
    RCC->CR |= (1 << 26); // enable PLLI2SON
    while(!(RCC->CR & (1 << 27))); // wait until PLLI2SRDY
}

/* Setup I2S for CS43L22 Audio DAC
 * Pins are connected to
 * PC7 - MCLK, PC10 - SCK, PC12 - SD, PA4 - WS
 */
void init_i2s3() {
    // Setup pins PC7 - MCLK, PC10 - SCK, PC12 - SD, PA4 - WS
    RCC->AHB1ENR |= (RCC_AHB1ENR_GPIOAEN | RCC_AHB1ENR_GPIOCEN);
    RCC->APB1ENR |= RCC_APB1ENR_SPI3EN;
    // PC7 alternate function mode MCLK
    GPIOC->MODER   &= ~(3U << 7*2);
    GPIOC->MODER   |= (2 << 7*2);
    GPIOC->OSPEEDR |= (3 << 7*2);
    GPIOC->AFR[0]  |= (6 << 7*4);
    // PC10 alternate function mode SCL
    GPIOC->MODER   &= ~(3U << 10*2);
    GPIOC->MODER   |= (2 << 10*2);
    GPIOC->OSPEEDR |= (3 << 10*2);
    GPIOC->AFR[1]  |= (6 << (10-8)*4);
    // PC12 alternate function mode SD
    GPIOC->MODER   &= ~(3U << 12*2);
    GPIOC->MODER   |= (2 << 12*2);
    GPIOC->OSPEEDR |= (3 << 12*2);
    GPIOC->AFR[1]  |= (6 << (12-8)*4);
    // PA4 alternate function mode WS
    GPIOA->MODER   &= ~(3U << 4*2);
    GPIOA->MODER   |= (2 << 4*2);
    GPIOA->OSPEEDR |= (3 << 4*2);
    GPIOA->AFR[0]  |= (6 << 4*4);

    // Configure I2S
    SPI3->I2SCFGR = 0; // reset registers
    SPI3->I2SPR   = 0; // reset registers
    SPI3->I2SCFGR |= (1 << 11); // I2S mode is selected
    // I2S config mode
    // 10 - Master Transmit
    // 11 - Master Receive
    // Since we will just use built-in beep, we can set it up as receive
    // mode to always activate clock.
    SPI3->I2SCFGR |= (3 << 8);

    // have no effect
    SPI3->I2SCFGR |= (0 << 7);  // PCM frame sync, 0 - short frame
    SPI3->I2SCFGR |= (0 << 4);  // I2S standard select, 00 Philips standard, 11 PCM standard
    SPI3->I2SCFGR |= (0 << 3);  // Steady state clock polarity, 0 - low, 1 - high
    SPI3->I2SCFGR |= (0 << 0);  // Channel length, 0 - 16bit, 1 - 32bit

    SPI3->I2SPR |= (1 << 9); // Master clock output enable
    // 48 Khz
    SPI3->I2SPR |= (1 << 8); // Odd factor for the prescaler (I2SODD)
    SPI3->I2SPR |= (3 << 0); // Linear prescaler (I2SDIV)

    SPI3->I2SCFGR |= (1 << 10); // I2S enabled
}

/*************************************************
* CS43L22 related functions
*************************************************/

void init_cs43l22(uint8_t an_ch) {
    // setup reset pin for CS43L22 - GPIOD 4
    RCC->AHB1ENR |= RCC_AHB1ENR_GPIODEN;
    GPIOD->MODER &= ~(3U << 4*2);
    GPIOD->MODER |=  (1 << 4*2);
    // activate CS43L22
    GPIOD->ODR   |=  (1 << 4);

    uint8_t data;
    // power off
    i2c_write(CS43L22_REG_POWER_CTL1, CS43L22_PWR_CTRL1_POWER_DOWN);

    // headphones on, speakers off
    data = (2 << 6) | (2 << 4) | (3 << 2) | (3 << 0);
    i2c_write(CS43L22_REG_POWER_CTL2, data);

    // auto detect clock
    data = (1 << 7);
    i2c_write(CS43L22_REG_CLOCKING_CTL, data);

    // slave mode, DSP mode disabled, I2S data format, 16-bit data
    data = (1 << 2) | (3 << 0);
    i2c_write(CS43L22_REG_INTERFACE_CTL1, data);

    // select ANx as passthrough source based on the parameter passed
    if ((an_ch > 0) && (an_ch < 5)) {
        data = (uint8_t)(1 << (an_ch-1));
    }
    else {
        data = 0;
    }
    i2c_write(CS43L22_REG_PASSTHR_A_SELECT, data);
    i2c_write(CS43L22_REG_PASSTHR_B_SELECT, data);

    // ganged control of both channels
    data = (1 << 7);
    i2c_write(CS43L22_REG_PASSTHR_GANG_CTL, data);

    // playback control 1
    // hp gain 0.6, single control, master playback
    data = (3 << 5) | (1 << 4);
    i2c_write(CS43L22_REG_PLAYBACK_CTL1, data);

    // misc controls,
    // passthrough analog enable/disable
    // passthrough mute/unmute
    if ((an_ch > 0) && (an_ch < 5)) {
        data = (1 << 7) | (1 << 6);
    }
    else {
        data = (1 << 5) | (1 << 4); // mute
    }
    i2c_write(CS43L22_REG_MISC_CTL, data);

    // passthrough volume
    data = 0; // 0 dB
    i2c_write(CS43L22_REG_PASSTHR_A_VOL, data);
    i2c_write(CS43L22_REG_PASSTHR_B_VOL, data);

    // pcm volume
    data = 0; // 0 dB
    i2c_write(CS43L22_REG_PCMA_VOL, data);
    i2c_write(CS43L22_REG_PCMB_VOL, data);

    start_cs43l22();
}

void start_cs43l22() {
    // initialization sequence from the data sheet pg 32
    // write 0x99 to register 0x00
    i2c_write(0x00, 0x99);
    // write 0x80 to register 0x47
    i2c_write(0x47, 0x80);
    // write 1 to bit 7 in register 0x32
    uint8_t data = i2c_read(0x32);
    data |= (1 << 7);
    i2c_write(0x32, data);
    // write 0 to bit 7 in register 0x32
    data &= (uint8_t)(~(1U << 7));
    i2c_write(0x32, data);
    // write 0x00 to register 0x00
    i2c_write(0, 0x00);

    // power on
    i2c_write(CS43L22_REG_POWER_CTL1, CS43L22_PWR_CTRL1_POWER_UP);
    // wait little bit
    for (volatile int i=0; i<500000; i++);
}

void I2C1_ER_IRQHandler(){
    // error handler
    GPIOD->ODR |= (1 << 14); // red LED
}
/*************************************************
* systik functions
*************************************************/
void SysTick_Handler(void)
{
    if (tDelay != 0)
    {
        tDelay--;
    }
}

void init_systick(uint32_t s)
{
    // Clear CTRL register
    SysTick->CTRL = 0x00000;
    // Main clock source is running with HSI by default which is at 8 Mhz.
    // SysTick clock source can be set with CTRL register (Bit 2)
    // 0: Processor clock/8 (AHB/8)
    // 1: Processor clock (AHB)
    SysTick->CTRL |= (1 << 2);
    // Enable callback (bit 1)
    SysTick->CTRL |= (1 << 1);
    // Load the value
    SysTick->LOAD = (uint32_t)(s-1);
    // Set the current value to 0
    SysTick->VAL = 0;
    // Enable SysTick (bit 0)
    SysTick->CTRL |= (1 << 0);
}


void TIM2_IRQHandler(void) //refresh the charge on PC7 every .3 seconds
{
    if (TIM2->DIER & 0x01) {
        if (TIM2->SR & 0x01) {
            TIM2->SR &= ~(1U << 0);
        }
    }
    if(myState == FIRST_PRESS_DEBOUNCE)
    {
        myState = LONG_PRESS;
        GPIOD->ODR ^= (1 << 15); // toggle blue led
        playing ^= 1; //toggle paused
    }
}

void TIM3_IRQHandler(void) //refresh the charge on PC7 every .3 seconds
{
    
    // clear interrupt status
    if (TIM3->DIER & 0x01) {
        if (TIM3->SR & 0x01) {
            TIM3->SR &= ~(1U << 0);
        }
    }
    //GPIOD->ODR ^= (1 << 12);
    if(myState == FIRST_UNPRESS_DEBOUNCE)
    {
        myState = FIRST_PRESS_DONE;
        TIM4->CR1 |= (1 << 0);
        GPIOD->ODR ^= (1 << 12); // toggle green led
        if(songChoice == MARY)
        {
            mary_note = note;
            mary_speed = speed;
            note = twinkle_note;
            len = TWINKLE_LEN;
            song = twinkle;
            tempo = twinkleTempo;
            speed = twinkle_speed;
            songChoice = TWINKLE;
        } 
        else
        {
            twinkle_note = note;
            twinkle_speed = speed;
            note = mary_note;
            len = MARY_LEN;
            song = mary;
            tempo = maryTempo;
            speed = mary_speed;
            songChoice = MARY;
        }

    }
}

void TIM4_IRQHandler(void) //refresh the charge on PC7 every .3 seconds
{
    
    // clear interrupt status
    if (TIM4->DIER & 0x01) {
        if (TIM4->SR & 0x01) {
            TIM4->SR &= ~(1U << 0);
        }
    }
    //GPIOD->ODR ^= (1 << 13);
    if (myState == FIRST_PRESS)
    {
        myState = FIRST_PRESS_DEBOUNCE;
    }
    else if (myState == SECOND_PRESS)
    {
        myState = SECOND_PRESS_DEBOUNCE;
    }
    else if (myState == FIRST_PRESS_DONE)
    {
        myState = NO_PRESS;
    }
    else if (myState == LONG_PRESS_DONE)
    {
        myState = NO_PRESS;
    }
    else if (myState == FIRST_UNPRESS)
    {
        myState = FIRST_UNPRESS_DEBOUNCE;
    }
}

void EXTI0_IRQHandler(void)
{

    // Check if the interrupt came from exti0
    if (EXTI->PR & (1 << 0))
    {
        EXTI->PR = (1 << 0); //set a timer, if it's not called before timer is done --> long press
        if(myState == NO_PRESS)
        {
            TIM2->CR1 |= (1 << 0);
            TIM4->CR1 |= (1 << 0);
            myState = FIRST_PRESS;
        }
        else if(myState == FIRST_PRESS_DEBOUNCE)
        {
            myState = FIRST_UNPRESS;
            TIM2->CR1 &= ~(1U << 0);
            TIM2->CNT = 0;
            TIM3->CR1 |= (1 << 0);
            TIM4->CR1 |= (1 << 0);
        }
        else if (myState == LONG_PRESS)
        {
            myState = LONG_PRESS_DONE;
            TIM4->CR1 |= (1 << 0);
        }
        else if (myState == FIRST_UNPRESS_DEBOUNCE)
        {
            GPIOD->ODR ^= (1 << 14);
            myState = SECOND_PRESS;
            TIM4->CR1 |= (1 << 0);

            if(speed == 4)
            {
                speed = 1;
            }
            else
            {
                speed *= 2;
            }
        }
        else if (myState == SECOND_PRESS_DEBOUNCE)
        {
            myState = NO_PRESS;
        }
        
    }

    return;
}

/*************************************************
* main code starts from here
*************************************************/
int main(void)
{
    /* set system clock to 168 Mhz */
    set_sysclk_to_100();

    //systik set up
      // SystemCoreClock should be configured correctly
    // depending on the operating frequency
    // SysTick runs at the same speed, so if we generate
    // a tick every clock_speed/1000 cycles, we can generate
    // a 1 ms tick speed.
    init_systick(SystemCoreClock/1000);

    //*******************************
    // setup LEDs - GPIOD 12,13,14,15
    //*******************************
    RCC->AHB1ENR |= RCC_AHB1ENR_GPIODEN;
    GPIOD->MODER &= ~(0xFFU << 24);
    GPIOD->MODER |= (0x55 << 24);
    GPIOD->ODR    = 0x0000;

    //*******************************
    // setup I2C - GPIOB 6, 9
    //*******************************
    // enable I2C clock
    RCC->APB1ENR |= RCC_APB1ENR_I2C1EN;

    // setup I2C pins
    RCC->AHB1ENR |= RCC_AHB1ENR_GPIOBEN;
    GPIOB->MODER &= ~(3U << 6*2); // PB6
    GPIOB->MODER |=  (2 << 6*2); // AF
    GPIOB->OTYPER |= (1 << 6);   // open-drain
    GPIOB->MODER &= ~(3U << 9*2); // PB9
    GPIOB->MODER |=  (2 << 9*2); // AF
    GPIOB->OTYPER |= (1 << 9);   // open-drain

    // choose AF4 for I2C1 in Alternate Function registers
    GPIOB->AFR[0] |= (4 << 6*4);     // for pin 6
    GPIOB->AFR[1] |= (4 << (9-8)*4); // for pin 9

    // reset and clear reg
    I2C1->CR1 = I2C_CR1_SWRST;
    I2C1->CR1 = 0;

    I2C1->CR2 |= (I2C_CR2_ITERREN); // enable error interrupt

    // fPCLK1 must be at least 2 Mhz for SM mode
    //        must be at least 4 Mhz for FM mode
    //        must be multiple of 10Mhz to reach 400 kHz
    // DAC works at 100 khz (SM mode)
    // For SM Mode:
    //    Thigh = CCR * TPCLK1
    //    Tlow  = CCR * TPCLK1
    // So to generate 100 kHz SCL frequency
    // we need 1/100kz = 10us clock speed
    // Thigh and Tlow needs to be 5us each
    // Let's pick fPCLK1 = 10Mhz, TPCLK1 = 1/10Mhz = 100ns
    // Thigh = CCR * TPCLK1 => 5us = CCR * 100ns
    // CCR = 50
    I2C1->CR2 |= (10 << 0); // 10Mhz periph clock
    I2C1->CCR |= (50 << 0);
    // Maximum rise time.
    // Calculation is (maximum_rise_time / fPCLK1) + 1
    // In SM mode maximum allowed SCL rise time is 1000ns
    // For TPCLK1 = 100ns => (1000ns / 100ns) + 1= 10 + 1 = 11
    I2C1->TRISE |= (11 << 0); // program TRISE to 11 for 100khz
    // set own address to 00 - not really used in master mode
    I2C1->OAR1 |= (0x00 << 1);
    I2C1->OAR1 |= (1 << 14); // bit 14 should be kept at 1 according to the datasheet

    // enable error interrupt from NVIC
    NVIC_SetPriority(I2C1_ER_IRQn, 1);
    NVIC_EnableIRQ(I2C1_ER_IRQn);

    I2C1->CR1 |= I2C_CR1_PE; // enable i2c

    // audio PLL
    init_i2s_pll();
    // audio out
    init_i2s3();
    // initialize audio dac
    init_cs43l22(0);

    // read Chip ID - first 5 bits of CHIP_ID_ADDR
    uint8_t ret = i2c_read(CS43L22_REG_ID);

    if ((ret >> 3) != CS43L22_CHIP_ID) {
        GPIOD->ODR |= (1 << 13); // orange led on error
    }

    // beep volume
    uint8_t vol = 7; //0x1C; // -6 - 12*2 dB
    i2c_write(CS43L22_REG_BEEP_VOL_OFF_TIME, vol);

    //headphone volume
    i2c_write(CS43L22_REG_HEADPHONE_A_VOL, 0xC1);

    // TIM2 SET UP
    // enable SYSCFG clock (APB2ENR: bit 14)
    RCC->APB2ENR |= (1 << 14);

    //TIM2 Set up
    // enable TIM2 clock (bit2)
    RCC->APB1ENR |= (1 << 0);

    TIM2->PSC = 4999; //set TIM3 prescalar
    TIM2->ARR = 10000; //set auto refil value to 1.1 seconds
    TIM2->CR1 |= (1 << 3);
    TIM2->DIER |= (1 << 0);  //enable TIM3 interrupt
    TIM2->CR1 |= (1 << 2);

    //Button set up
    
    // enable GPIOA clock (AHB1ENR: bit 0)
    RCC->AHB1ENR |= (1 << 0);
    GPIOA->MODER &= 0xFFFFFFFC;   // Reset bits 0-1 to clear old values
    GPIOA->MODER |= 0x00000000;   // Make button an input

    SYSCFG->EXTICR[0] |= 0x00000000; // Write 0000 to map PA0 to EXTI0

    EXTI->RTSR |= 0x00001;   // Enable rising edge trigger on EXTI0 (tells you when button is pressed)

    EXTI->FTSR |= 0x00001;  //Enable falling edge trigger (tells you when button is unpressed)

    // Mask the used external interrupt numbers.
    EXTI->IMR |= 0x00001;    // Mask EXTI0

    //TIM3 Set up
    // enable TIM3 clock (bit1)
    RCC->APB1ENR |= (1 << 1);

    TIM3->PSC = 4999; //set TIM3 prescalar
    TIM3->ARR = 3000; //set auto refil value to 0.3 seconds
    TIM3->CR1 |= (1 << 3);
    TIM3->DIER |= (1 << 0);  //enable TIM3 interrupt
    TIM3->CR1 |= (1 << 2);

    //TIM4 Set up
    RCC->APB1ENR |= (1 << 2);

    TIM4->PSC = 4999; //set TIM3 prescalar
    TIM4->ARR = 125; //set auto refil value to 0.3 seconds
    TIM4->DIER |= (1 << 0);  //enable TIM3 interrupt
    TIM4->CR1 |= (1 << 3);
    TIM4->CR1 |= (1 << 2);

    // Set Priority for each interrupt request
    NVIC_SetPriority(EXTI0_IRQn, 1); // Priority level 1
    // enable EXT0 IRQ from NVIC
    NVIC_EnableIRQ(EXTI0_IRQn);

    NVIC_SetPriority(TIM2_IRQn, 2); // Priority level 2
    // enable TIM2 IRQ from NVIC
    NVIC_EnableIRQ(TIM2_IRQn);

    NVIC_SetPriority(TIM3_IRQn, 3); // Priority level 2
    // enable TIM3 IRQ from NVIC
    NVIC_EnableIRQ(TIM3_IRQn);

    NVIC_SetPriority(TIM4_IRQn, 3); // Priority level 2
    // enable TIM3 IRQ from NVIC
    NVIC_EnableIRQ(TIM4_IRQn);

    // Enable Timer 2 and 3 module (CEN, bit0)
    TIM3->CR1 |= (1 << 0);
    TIM2->CR1 |= (1 << 0);
    TIM4->CR1 |= (1 << 0);
    //i2c_write(CS43L22_REG_BEEP_TONE_CFG, (1 << 6));
    //i2c_write(CS43L22_REG_BEEP_FREQ_ON_TIME, 0x31);
    
    while(1)
    {
        if(playing)  
        { 
            if(song[note])
            {
                i2c_write(CS43L22_REG_BEEP_TONE_CFG, 0x0);
                i2c_write(CS43L22_REG_BEEP_FREQ_ON_TIME, song[note] | getTempo(tempo[note], speed));
                i2c_write(CS43L22_REG_BEEP_TONE_CFG, (1 << 6));
            }
            if(note == len - 1) //changes array pos to next tone
            {
                note = 0;
            }
            else
            {
                note++;
            }
                
            delay_ms(getDelay(speed)); // 0.5 sec delay
        }
    }
    return 0;
}

/*
 * Millisecond delay function.
 */
void delay_ms(uint32_t s)
{
    tDelay = s;
    while(tDelay != 0);
}

uint32_t getDelay (uint32_t speed)
{
    if(speed == 1)
    {
        return 1000;
    }
    else if (speed == 2)
    {
        return 500;
    }
    else if (speed == 4)
    {
        return 250;
    }
    
}

uint32_t getTempo (uint8_t tempo, uint32_t speed)
{
    if(speed == 1)
    {
        if(tempo)
        {
            return (1 << 1);
        }
        else
        {
            return (1 << 0);
        }
    }
    else if (speed == 2)
    {
        return tempo;
    }
    else if (speed == 4)
    {
        return 0;
    }
    
}

 
